# Fucktorio Project Audit Plan

**Created:** 2026-04-24
**Issue:** #174 — `feat(misia)` — welcome audit & evaluation plan

---

## 1. Project Overview

Fucktorio is a Factorio factory blueprint generator that takes a target item + production rate, solves recipe dependencies, generates a spatial bus layout, and exports a Factorio-importable blueprint string. It runs entirely client-side via WASM in the browser.

**Repository:** `storkme/fucktorio`
**Language:** Rust (workspace) + TypeScript (web app)
**Test suite:** 376 unit tests, 9 passing e2e tests, 19 ignored (known failures)
**Total source:** ~42,744 lines of Rust across 44 files

---

## 2. Architecture Assessment

### 2.1 Workspace Structure (3 crates + web)

| Crate | Lines | Role | Health |
|-------|-------|------|--------|
| `crates/core/` | ~35,000 | Pure shared logic: solver, recipe DB, bus layout engine, A*, SAT crossing solver, blueprint export, validation (23 checks) | ✅ Core pipeline solid |
| `crates/wasm-bindings/` | ~330 | Thin `wasm-bindgen` wrapper exposing pipeline to browser | ✅ Thin, well-factored |
| `crates/mining-cli/` | ~320 | `blueprint-analyze` binary for community blueprint dissection | ✅ Focused tooling |
| `web/` | ~55,000 (TS) | Vite + PixiJS v8 + pixi-viewport UI | ⚠️ Large, needs triage |

### 2.2 Pipeline Stages (4 phases)

```
Solver → Bus Layout → Blueprint Export → Validation
```

All four stages are implemented in Rust with clear module boundaries. The pipeline is deterministic and single-pass (no retry loops, no mode switches).

---

## 3. Evaluation Against Stated Objectives

### Objective 1: "You pick the target item and a production rate"
**Status:** ✅ Fully implemented
- Web UI item picker with searchable list
- Rate input with slider
- Machine tier selector (AM1/2/3, chemical plant, electric furnace, etc.)
- Belt tier selector (yellow/red/blue)
- External input checkboxes
- URL state encodes all parameters for shareable links

### Objective 2: "Fucktorio resolves the recipe tree"
**Status:** ✅ Implemented with known gaps
- Recursive recipe resolution with cycle detection
- Space Age DLC machine category mappings in `recipe_db.rs`
- Recipe exclusions supported (e.g., skip oil chain for plastic-bar)
- **Gap:** No module-aware machine counts (solver uses base crafting speed)
- **Gap:** No support for kovarex (self-feeding cyclic recipe)

### Objective 3: "Places machines + belts + pipes + power"
**Status:** ⚠️ Partially implemented
- Bus layout engine: rows of machines with trunk belts
- Ghost A* router for connecting belts
- SAT crossing-zone solver for perpendicular belt crossings
- Junction solver (region-growth + SAT fallback)
- Power pole placement with connectivity repair
- Fluid pipe routing
- **Gap:** Space Age non-square machines (recycler 2×4, crusher 2×3) not handled
- **Gap:** Beacons not placed in layouts (analysis can parse them, solver cannot generate them)
- **Gap:** Kovarex needs belt loops for U-235 recirculation

### Objective 4: "Validates the result against Factorio's physics"
**Status:** ✅ Implemented — 23 functional checks
- Belt connectivity, flow paths, reachability
- Entity overlaps, belt loops, dead ends, item isolation
- Underground belt pairs and sideloading
- Inserter chains and direction
- Fluid port connectivity and pipe isolation
- Power coverage and pole network connectivity
- Lane throughput and input rate delivery
- Parallelized with rayon for performance

### Objective 5: "Emits an importable blueprint string"
**Status:** ✅ Implemented
- Direct JSON + zlib + base64 encoding (no draftsman dependency)
- Round-trip tested in e2e suite
- Space Age `mirror` field supported
- Blueprint parser handles 1.x and 2.0 formats including books

---

## 4. Recipe Complexity Ladder — Current State

| Tier | Recipe | Status | Known Issues |
|------|--------|--------|--------------|
| 1 | `iron-gear-wheel` | ✅ Solved | None |
| 2 | `electronic-circuit` | ⚠️ Partial | `tier2_electronic_circuit` ignored (SAT routing bug); `from_ore` variant passes |
| 3 | `plastic-bar` | ✅ Solved | `from_crude` ignored (ghost-mode routing fails) |
| 4 | `advanced-circuit` | ❌ Not working | Lane-throughput warnings (#64), ore chain blocked (#68), missing balancer shapes (#136) |
| 5 | `processing-unit` | ❌ Not attempted | Deep chain, multiple fluids |
| 6 | `rocket-control-unit` | ❌ Not attempted | Very deep chain |

**Passing e2e tests:** 9/28 (32%)
**Ignored e2e tests:** 19/28 (68%) — most are known failures

---

## 5. Known Issues & Open Tracking

### Critical (block higher tiers)
1. **#136** — Missing 5→9 balancer template crashes layout
2. **#135** — Balancer templates oversized — main compaction lever
3. **#68** — Bus placer: 2+ solid + 1 fluid input rows don't fit in 3-tile pitch
4. **#64** — Single-lane throughput bottleneck on intermediate trunks (CLOSED, but tier4 still has lane-throughput warnings)

### Medium (layout quality)
5. **#163** — Topology boundaries loses input-chain when immediate feeder enters zone
6. **#165** — Fluid trunks should use underground pipes for adjacency isolation
7. Ghost router allows paths ending on hard obstacles (`astar.rs:695`)
8. Power warning: disconnected poles in tier2-from-ore layouts

### Low (nice-to-have)
9. No module/beacon support in solver or layout
10. Kovarex (self-feeding recipe) not supported
11. Space Age non-square machines not fully supported
12. Web app uses colored rectangles instead of actual Factorio sprites

---

## 6. Code Quality Assessment

### 6.1 Strengths
- **Clear module boundaries** — each validation check is isolated, each pipeline phase has documented contracts (`docs/ghost-pipeline-contracts.md`)
- **Comprehensive testing** — 376 unit tests, 9 e2e tests, stress corpus with scoreboards
- **Trace-driven debugging** — 22 structured trace events, snapshot debugger, phase timing
- **Deterministic output** — JSON insertion order preserved, no randomness in layout
- **Well-documented** — CLAUDE.md, TASKS.md, ghost-pipeline-contracts.md, factorio-mechanics.md
- **Good error handling** — `thiserror` enums, `Result` types, graceful degradation

### 6.2 Areas for Improvement
- **Large files** — `ghost_router.rs` (3,288 lines), `balancer_library.rs` (4,370 lines), `belt_flow.rs` (3,789 lines), `templates.rs` (3,214 lines), `sat.rs` (2,864 lines). These should be split.
- **Ignored test count** — 19/28 e2e tests are `#[ignore]`. This is a signal that the test suite is not a reliable regression guard.
- **Dead code risk** — `sat.rs` marked as "retained but no longer used by route_bus_ghost". Needs cleanup or reintegration.
- **External dependency** — `Factorio-SAT` is vendored but not in the repo (needs manual clone). The `generate_balancer_library.py` script requires it on PATH.
- **Web app size** — `main.ts` is 47KB, making it the largest single file. Needs modularization.

### 6.3 TODOs in Code
- `blueprint.rs` has a TODO about multi-tile entity positioning for 3×3 assemblers
- `TASKS.md` has 40+ open items across 10 categories (Visual Interactivity, Factory Footprint, Layout Improvements, Space Age, Kovarex, Debugging, Modules & Beacons, Blueprint Mining)

---

## 7. Recommended Audit Plan

### Phase 1: Baseline Verification (Week 1)
**Goal:** Confirm the 9 passing tests are reliable regression guards.

1. Run all 9 passing e2e tests with `FUCKTORIO_DUMP_SNAPSHOTS=1` to capture layout snapshots
2. Manually verify each snapshot in the web app — check for disconnected belts, missing inserters, power gaps
3. Add `assert_no_warnings` to tier1 tests (currently only tier2-from-ore allows power warnings)
4. Document which warnings are real vs. false positives

### Phase 2: Tier 2 Fix (Week 2-3)
**Goal:** Make `tier2_electronic_circuit` pass (un-ignore it).

1. Reproduce the ignored test failure — "SAT has an honest model... but SAT still satisfies iter 2 with a solution the reachability walker rejects"
2. Trace the failure through the ghost router — check `RouteFailure` events in trace
3. Fix the SAT routing or walker reasoning bug
4. Add the test back to the active suite

### Phase 3: Tier 3 Fix (Week 4)
**Goal:** Make `tier3_plastic_bar_from_crude` pass.

1. Ghost-mode routing fails on plastic-bar feeder paths from crude oil
2. Likely a fluid pipe routing issue — check `check_fluid_port_connectivity`
3. Fix and un-ignore

### Phase 4: Tier 4 Foundation (Week 5-6)
**Goal:** Remove lane-throughput warnings for advanced-circuit.

1. Address #64 — single-lane throughput bottleneck
2. Address #136 — missing balancer templates
3. Generate additional balancer shapes via `generate_balancer_library.py`
4. Fix the lane-planning logic for multi-lane families

### Phase 5: Space Age Support (Week 7-8)
**Goal:** Layout engine handles all Space Age machines.

1. Fix `machine_size()` for non-square machines (recycler, crusher)
2. Fix `row_kind()` and pitch calculation for 4×4 and 5×5 machines
3. Add Space Age machines to `MACHINE_ENTITY_NAMES` if missing
4. Test with Space Age recipes (e.g., `processing-unit` if applicable)

### Phase 6: Module & Beacon Support (Week 9-10)
**Goal:** Solver and layout engine support modules and beacons.

1. Add `ModuleConfig` parameter to `solve()`
2. Adjust machine count formula for module speed/productivity bonuses
3. Implement beacon row templates in `templates.rs`
4. Wire through WASM bindings and web UI

### Phase 7: Kovarex (Week 11-12)
**Goal:** Handle self-feeding cyclic recipes.

1. Research community kovarex strategies
2. Implement cycle detection and net-positive loop handling in solver
3. Design belt-loop layout for U-235 recirculation
4. Test with uranium processing chain

### Phase 8: Code Health (Ongoing)
**Goal:** Reduce technical debt.

1. Split large files (`ghost_router.rs`, `balancer_library.rs`, `belt_flow.rs`)
2. Clean up dead code in `sat.rs`
3. Add `assert_no_warnings` to tier1 tests
4. Reduce ignored test count from 19 to ≤5

---

## 8. Success Metrics

| Metric | Current | Target (12 weeks) |
|--------|---------|-------------------|
| Passing e2e tests | 9/28 (32%) | 20/28 (71%) |
| Ignored e2e tests | 19 | ≤5 |
| Tier ladder | Tier 3 solved | Tier 4 solved |
| Unit tests | 376 | 400+ |
| Large files (>2000 LOC) | 5 | ≤3 |
| Open critical issues | 4 | 0 |

---

## 9. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| SAT solver complexity | Tier 4 fix may require SAT solver changes | Start with lane-planning fix first (no SAT changes) |
| Factorio-SAT external dependency | Cannot regenerate balancer library without it | Vendor it in the repo or use pre-generated templates |
| Web app size | Hard to maintain and debug | Modularize `main.ts`, add entity hover tooltips |
| Space Age recipe data | `recipes.json` may need updates | Verify data extraction script works with Space Age |

---

## 10. Immediate Next Steps

1. **Run the full e2e suite** to confirm baseline (already done: 9 pass, 19 ignored)
2. **Pick one passing test** and add `assert_no_warnings` to find real vs. false-positive warnings
3. **Pick one ignored test** (preferably `tier2_electronic_circuit`) and investigate the failure
4. **Start Phase 1** of the audit plan: baseline verification

---

*This plan is a living document. Refine as we dig in.*
