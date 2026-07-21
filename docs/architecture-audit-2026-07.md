# Architecture audit — 2026-07-21

**Status:** investigation complete; findings captured. Nothing here is fixed yet except
where noted. This is a Notes-class document (session artifact) — mine it into RFCs /
followups docs / issues as items get picked up, then archive.

**Provenance:** full-repo deep review (solver, bus layout engine, validation/export,
web/tests/docs) by subagent sweep, followed by per-claim verification and three
targeted triage investigations (merge-tap flip, solver fidelity gaps, headless
Factorio verification). All file:line citations verified against `main@d468c68`.

---

## 1. Executive summary

The pipeline is a **greedy constructive heuristic with local repair**: LP solve →
deterministic single-shape bus layout (negotiated-congestion A* + region-growth
junction solver with SAT fallback) → blueprint export → 34-check validation. The
engineering culture (RFCs with kill criteria, decision logs, stress goldens, trace
events) is excellent. The findings below concern the *design's ceiling*:

1. **The confidence chain bottoms out at self-consistency, not the game.** There is
   no end-to-end mass-balance check; every rate-shaped validator check is a Warning;
   `carries` labels are the engine grading its own homework; the in-game import
   anchors are user-run and open. The headless-Factorio harness (§8.3) is feasible
   at 2–4 days and retires every open in-game kill criterion in one stroke.
2. **The layout engine re-derived VLSI CAD without adopting its mature results** —
   no rip-up of trunks, no placement perturbation, shallow repair budgets everywhere
   (1 layout retry, 5 growth iters, 64 region tiles, 700 SAT vars, 8 negotiation
   iters, 20 pole-repair iterations).
3. **The known root architectural mistake (1-trunk-per-consumer-row) has its fix
   built but not flipped** — merge-tap is fallback-only, gated on unfunded priced
   work the RFC decision log enumerates (§8.1).
4. **Parallel truths everywhere** (dual occupancy sets, two lane-rate walkers,
   triplicated lane capacity, hand-mirrored placer↔template geometry, hardcoded
   tables duplicating recipes.json data) — each a drift hazard with historical bugs
   cited inline.

---

## 2. Techniques employed (as-found)

| Stage | Technique | Key files |
|---|---|---|
| Solver | microlp simplex over recipe columns; frozen 5-number cost table; demand-closure + surplus-compression anti-gaming guards; Tarjan-SCC cycle refusals with ≤8-retry exclusion; byproducts as surplus variables; tree walk retained only as parity oracle | `netflow.rs`, `solver.rs` |
| Layout | Rows stacked south in solid-dependency topo order; vertical trunks west, taps east; "ghost" A* (paths may overlap, crossings repaired later); negotiated congestion (PathFinder-style, history/present penalties); region-growth junction solver + Varisat SAT fallback ladder; 4-candidate decomposition search; one +1-tile retry | `bus/layout.rs`, `bus/ghost_router.rs`, `bus/junction_solver.rs`, `sat.rs` |
| Export | Direct 2.0 envelope (zlib+base64 JSON); wires; quality; mirror; BlueprintInsertPlan modules | `blueprint.rs` |
| Validation | 34 checks; exact on static structure (overlaps, UG pairing, pipe isolation, port geometry, wire graph); heuristic on flow (two independent steady-state lane-rate walkers) | `validate/` |

---

## 3. Weak assumptions

### 3.1 Solver layer

1. **Default path ignores built-in machine productivity.** Foundry/biochamber/EMP
   `base_effect_productivity: 0.5` credited only when a module policy is active
   (`module_policy.rs:121-123,170`). Default SA solves plan up to 1.5× too many
   machines. *Deliberate deferral* (RFC-044 rev 2 non-goal, recorded followup at
   `recipe_db.rs:100-106`) — triaged in §8.2 GAP 1.
2. **Machine selection is a category table, not an optimization** — LP never
   compares foundry vs furnace (`recipe_db.rs:330-385`). Triaged §8.2 GAP 3.
3. **Available inputs are prices, not guarantees** (`w_available=1e-4`,
   `netflow.rs:61,671-703`): the LP produces declared-available items when the
   ingredient basket prices lower (copper-cable arithmetic: import 1e-4 vs produce
   5.04e-5). Triaged §8.2 GAP 4.
4. **Hard refusals on whole strategy classes.** Any fluid self-loop (coal
   liquefaction) and all multi-recipe cycles error out; the 8-retry budget is
   shared with the surplus-compression guard, so a pathological case can surface
   the wrong refusal (`netflow.rs:351-388,1063-1087`).
5. **Unknown machines silently solve at speed 1.0** — `MissingCraftingSpeed` is
   documented but unreachable (`recipe_db.rs:192-199,270-272`; fires only on
   speed ≤ 0, `solver.rs:394-399`). Triaged §8.2 GAP 2.
6. **Frozen 5-number cost table** with water special-cased *by name* at 0.01 —
   100× more expensive than a user-declared input (`netflow.rs:35-67,702-707`).
   Any revision needs an RFC decision-log entry (KC3).
7. **Factory boundary assumptions**: no mining/pumpjacks/offshore pumps/asteroid
   crushing, no spoilage, no fluid temperatures, no biochamber fuel, no power
   generation, probabilistic recipes expectation-only. "From ore" fixtures assume
   an unmodeled, possibly infeasible upstream.

### 3.2 Layout layer

8. **One bus shape, forever.** The junction solver can only grow a bbox and
   re-stamp belts — it can never ask the lane planner to swap columns or widen one
   gap. Geometry-level conflicts get belt-level fixes; the single geometry retry
   is global, +1 tile, once, then ships anyway with `ReactivePassNotConverged`
   (`layout.rs:291-309`).
9. **1-trunk-per-consumer-row split rule** (`lane_planner.rs:716`) — fan-out scales
   with consumer count, manufacturing unstampable shapes (15,14)/(12,7)/(8,19).
   Fix exists (merge-tap), fallback-only. Triaged §8.1.
10. **Placer↔template geometry hand-mirrored.** `input_belt_y`/`output_belt_y`
    re-derived per template arm in `placer.rs` by duplicating `templates.rs`
    internal constants; comments admit past dead-end bugs from exactly this. A
    "templates return their port geometry" contract deletes the bug class.
11. **Triplicated constants.** Lane capacity in `common.rs:331-336`,
    `lane_planner.rs:19-23`, *and* `partitioner.rs:41-45` (explicit "single source
    of truth is awkward" comment). UG reach conventions disagree in comments
    (4/6/8 vs 5/7/9 — see doc-rot L). Dual `Occupancy` vs `hard`/`existing_belts`
    bookkeeping synced only by `debug_assert!` — invisible drift in release/WASM
    (`ghost_router.rs:1409-1435,2303-2312`).
12. **Fluid trunks are a 5-special-case heuristic chain with no backtracking**,
    while belts get A* + SAT + negotiation. Every fluid failure degrades to "leave
    it broken, tell the validator" (`ghost_router.rs:840-1091`). Belt×pipe
    crossings forced into singleton clusters — never joint-solved
    (`ghost_router.rs:4031-4047`); `rfc-pipe-belt-junctions.md` self-described
    unfinished.
13. **Zero-margin tuned constants.** `SUBSTATION_BAND_TILES=2` lands covering
    poles at distance *exactly* 3 — one extra belt row re-breaks power
    (`layout.rs:431-446`). Pole-band thinning structurally unreachable at Normal
    (`layout.rs:1228-1238`). Pole repair caps at 20 iterations, silently gives up.
14. **Output merger: one belt per item, O(n) rows deep.** 60/s request on a 45/s
    belt stamps `rate=60.0` with zero validation issues — the lane walker never
    visits merger segments (#311). The south-flush correction
    (`ghost_router.rs:3545-3603`) exists only to patch the design against its own
    validator's dead-end semantics.

### 3.3 Validation layer — where "0 errors" can lie

15. **No end-to-end mass-balance check anywhere.** Nothing sums the target item
    reaching the layout edge vs requested rate. `assert_produces` in e2e is
    machine-count arithmetic that never looks at a belt (`analysis.rs:405-444`).
16. **Every rate-shaped check is a Warning** (inserter throughput, lane
    throughput, input-rate delivery, power coverage). "SOLVED / 0 errors" can
    coexist with starving machines.
17. **`carries` labels are circular ground truth** — item isolation, inserter
    conflict, per-item throughput, pipe isolation, fluid connectivity all trust
    the engine's own non-exported annotation. A router mislabel produces a
    self-consistent lie.
18. **Two lane-rate walkers that disagree.** `belt_flow`'s Jacobi phase fully
    mixes lanes at splitters; `belt_structural` preserves them; the former's
    comment contradicts rule S4 in `factorio-mechanics.md`. `input_priority`
    exported but never modeled. Give-up paths (3-retry sibling waits, exhausted
    convergence budgets) emit plausible numbers, no issue — `forward_converged`
    recorded and discarded (`belt_flow.rs:2860-3000`).
19. **Fluid identity at ports unchecked.** Swapped water/crude hookup on
    advanced-oil-processing validates clean; ≥1 input port suffices (dry second
    port passes); fluid throughput unmodeled entirely
    (`fluids.rs:470-488`).
20. **In-game anchors open** (rfc-inserter-sizing KC5, rfc-build-quality,
    merge-tap KC5) — user-run. See §8.3 for the fix.

### 3.4 Data & verification infrastructure

21. **recipes.json is a hand-augmented hybrid** — default draftsman extraction +
    surgically-appended recycling recipes; naive regeneration drops the appended
    ones (`scripts/extract_factorio_data.py:155-169`). Module slots live in a
    second hardcoded Rust table (`common.rs:1066-1086`). Drift vs game data is
    manual.
22. **Goldens are host-local** (SAT zone-cache + wall-clock-budget shaped),
    opt-in, not CI-enforced; CI replays a pinned cache snapshot.
23. **Stale docs** — see the doc-rot register (§7).

---

## 4. Cross-cutting themes

- **Parallel truths** — every duplicated source of truth has at least one
  historical bug cited inline. Uniform fix pattern: single source + one consumer
  contract.
- **Determinism as a straitjacket** — fixed tie-breaks and no random restarts
  rule out stochastic optimization, but *seeded* determinism is fully compatible
  with golden hashes. The constraint is over-broad.
- **Shallow repair budgets everywhere** — each cap individually tuned;
  collectively hard instances fail softly in many places at once, surfacing as
  warnings + trace events rather than a coherent "here is the minimal thing I
  couldn't do."
- **VLSI reinvention** — negotiated-congestion A* ≈ PathFinder, eviction ≈
  rip-up-and-reroute, region growth ≈ detailed-routing spill — without rip-up of
  *trunks*, placement perturbation, history decay, or length/area-aware costs.

---

## 5. Missed opportunities (ranked by leverage/effort)

1. **Headless-Factorio verification harness** — feasible, 2–4 days, retires all
   open in-game kill criteria. Full feasibility report: §8.3.
2. **End-to-end rate audit as a first-class check** — cheap proxy of #1: sum
   target item reaching the layout edge; plus port-fluid identity check,
   wrong-item-feed check (currently *skipped* when `required=0`,
   `belt_flow.rs:3300-3303`), dry-port check.
3. **Merge-tap default flip** — the RFC's Phase 2. NOT a small PR; triaged §8.1.
4. **Max-flow feasibility instead of heuristic walkers** — belt graph with vertex
   capacities is a textbook max-flow instance; exact cheap upper bound, min-cut
   as bottleneck diagnostic. Complements the walkers.
5. **Solver fidelity trio** — base productivity on None path (§8.2 GAP 1),
   unknown-machine hard error (GAP 2), strict-available-inputs option (GAP 4).
6. **Templates return their geometry** — one `RowGeometry` contract consumed by
   placer, lane planner, validator. Deletes the mirrored-constants bug class.
7. **Fluid routing through the belt A*/occupancy machinery**, with joint
   belt×pipe junction clusters.
8. **Proactive power reservation** — reserve pole corridors in occupancy before
   routing instead of fitting poles into leftovers with a zero-margin +2 band.
9. **Beacons** — the endgame Factorio design pattern is entirely absent
   (`BEACON_DISTRIBUTION_EFFECTIVITY` is a 1.x fudge used only by the analyzer,
   `common.rs:1116-1120`). A beaconed row template *simplifies* layout (fewer
   machines/inserters/rows) and matches real megabases.
10. **Seeded stochastic search as a candidate dimension** — the 4-candidate
    decomposition search already exists; add seeded lane-order/retry variants.
    Determinism preserved (seed in URL).
11. **Data pipeline hardening** — regenerate recipes.json from pinned draftsman
    with the augment as a *patch file*; move module slots + belt/lane capacities
    into the generated table; unknown machine names → hard error (GAP 2).

---

## 6. Alternative approaches to the same problem

**A. City-block / cell composition (template-first).** Pre-verified cells (one per
recipe × machine tier × beacon config) with fixed footprints and contractual edge
ports; composition becomes block-level place-and-route. The bus engine shrinks to
a corridor router; validation shrinks to port-contract checking; in-game
verification is per-cell and amortized. Lose optimality, gain robustness by an
order of magnitude. The balancer library is already this idea at splitter scale —
apply it at recipe scale.

**B. Full VLSI flow.** Commit to the analogy: min-cut/force-directed placement of
rows (crossing+length jointly), global routing on a coarse grid, negotiated
detailed routing with rip-up of *everything including trunks* and history decay.
The literature (PathFinder, VPR) solves the convergence problems currently hit
with 8-iteration caps. "Make the current architecture adult" path.

**C. Constraint-based global layout (CP-SAT/SMT).** The balancer-synthesis spike
failed at (n,m)-template scale, but the honest reading is that *unconstrained*
synthesis is infeasible, not that constraint solving is. Best fit: escalating
fallback for junction zones **with splitters in the encoding** (currently excluded
by construction, `sat.rs:8`) and per-row micro-optimization. Not viable
whole-factory at USP scale.

**D. Simulator-in-the-loop search.** Black-box optimization over the parameterized
family (lane order, gaps, split rules) with validators as fitness; seeded
annealing/CMA-ES over the existing candidate set. Strong as a *wrapper* on A–C.

**E. Interactive repair instead of one-shot generation.** Keep the constructive
engine; expose the IR for user edits (move a row, pin a lane) + localized repair.
The F2 SAT-zone editor is an embryonic version. Matches how the tool is actually
used (URLs, eyeballing, iteration).

**F. The game as the oracle** — §8.3. Not an alternative so much as the missing
ground truth every other approach needs.

**Assessment:** A and F are the highest-value directions not being taken; B is the
natural maturation of what exists; C worth one scoped spike (junctions with
splitters); D/E are wrappers. The current architecture is not wrong — its failures
have become *interaction* failures (geometry vs belts vs power vs fluids), which
is what feedback loops (B) and coarser abstraction (A) exist to solve.

---

## 7. Doc-rot register (verified 2026-07-21)

Every claim below was verified against the code by a dedicated sweep. **A–J, L–Q
are doc/comment fixes** (safe). **K and O are code changes** (dead code removal —
needs test run). Verdicts: TRUE = stale as claimed.

**Status 2026-07-21:** A–J, L, N, P fixed on branch `docs/audit-doc-rot` (plus
two bonus finds: CLAUDE.md pipeline stage 1 "Recursively resolves", and a stale
`bus_router.rs` comment in `tests/e2e.rs`). M was already fixed on main by the
RFC-046 merge. K and O remain open (code changes, deferred to a follow-up PR).

| # | File:line | Problem | Fix |
|---|---|---|---|
| A | `CLAUDE.md:113,146,210` | "25 checks" — 34 dispatched (`validate/mod.rs:472-519`) | "34" ×3 |
| B | `CLAUDE.md:143` | astar.rs credited with `astar_path`+`negotiate_lanes` — only `ghost_astar` exists; those names exist nowhere in `crates/` | Reword; negotiation lives in `ghost_router.rs` |
| C | `CLAUDE.md:198` | Cites `RouteFailure`/`BridgeDropped` as emitted — declared (`trace.rs:638,734`) but **never emitted**; `CrossingZoneSkipped`/`BalancerStamped` OK | List actually-emitted events: `JunctionGrowthCapped`, `JunctionStrategyAttempt`, `GhostSpecFailed`, `TapBridgeUnbridgeable`, `LayoutRetried`…; note the two dead ones |
| D | `docs/ghost-pipeline-contracts.md:7,17-18,34-36,343,356` | "single-pass — no retry loop" (retry orchestrator exists, `layout.rs:144-340`); diagram shows poles before routing (poles are LAST, `layout.rs:732-744,938`); references deleted `bus_router.rs` ×3 (types live in `lane_planner.rs:30,130`) | Reword all |
| E | `solver.rs:82-85` | `solve()` docstring says "Recursively resolves" — routes to LP | Reword (net-flow LP, free selection default) |
| F | `solver.rs:221-223` | "opt-in until [fluid stagger gap] closes" — free mode IS the default everywhere | Drop the sentence |
| G | `netflow.rs:151-152` | "becomes the default in Phase 3" — Phase 3 shipped 2026-07-11 | Past tense |
| H | `netflow.rs:1036` | Points at `solver.rs:257` — ingredient loop is at `solver.rs:455` | Name the function, not the line |
| I | `recipe_db.rs:263-265` (+dup `:749`) | Claims unknown machines hit `MissingCraftingSpeed` — unreachable; they solve silently at 1.0 | Correct the contract text |
| J | `ghost_router.rs:3206-3211` | Warning says "(junction solver not yet implemented)" — it exists; means "strategies exhausted" | Reword warning. **Check test assertions on this string first** |
| K | `ghost_router.rs:2977-3030` | `proposed_tiles`/`ug_pair_interiors` computed, never used (release passes `None`, `:3090-3094`); "minimum-authority rule" comment describes abandoned behavior (#243) | CODE: delete dead block + stale comment |
| L | `junction_sat_strategy.rs:143-144` | "yellow=5, red=7, blue=9" — code uses `ug_max_reach` = 4/6/8 (`common.rs:338-344`) | Fix comment |
| M | `docs/rfcs.md:61-63` | Duplicated "Next number: RFC-046" / "RFC-045" | Delete stale line |
| N | `solver.rs:123-125` | Doc example excludes coal-liquefaction — a refused fluid self-loop, so vacuous on this path | Use a live example (e.g. exclude `basic-oil-processing`) |
| O | `validate/underground.rs` (548 lines) | Declared (`mod.rs:11`) but zero references — dispatcher uses belt_flow's UG checks. Also 6 dead duplicates in `belt_flow.rs` (`check_belt_throughput:1148`, `check_output_belt_coverage:1191`, `check_belt_dead_ends:1513`, `check_belt_loops:1634`, `check_belt_item_isolation:1690`, `check_belt_inserter_conflict:1750`); `check_lane_throughput:2111` is a LIVE split-brain duplicate (called from `bus/template_validate.rs:26,115`) | CODE: delete dead module + 6 dead functions; consolidate the split-brain walker separately |
| P | `docs/factorio-mechanics.md:106` (I5) | Says inserters drop on NEAR lane — code uses FAR lane (`common.rs:650-668`), which matches the game. The doc's own dot-product sentence describes far-lane geometry | near → far |
| Q | `CLAUDE.md:195` | "34 e2e" — now 76 `#[test]` fns, 56 active, plus parity harness (13) and more suites | Reword or drop the count |

---

## 8. Triage reports

### 8.1 Merge-tap default flip (RFC merge-tap-trunks Phase 2)

**Verdict: NOT a small PR — it is the RFC's Phase 2, a medium-to-large funded
phase, and the RFC itself says so.** Mechanical diff is tiny (default at
`layout.rs:126`; stampability conjunct at `lane_planner.rs:949-952`), but:

1. **Explicitly gated on unfunded, priced work** (decision log close-out
   2026-07-14): merge-tap-aware lane ordering ("days", the named gate on BOTH the
   utility@10/s residual AND the flip), junction-solver capability ("deep
   program"), balancer-width column reflow (`lane_planner.rs:451-457` — reserves
   lane *count*, never balancer *width*; 8 contamination errors on utility).
   Final log entry: "Phase 2 HOLDS here pending the user's funding decision."
2. **Knowingly regresses a green fixture by severity kind today.** EC@35s
   merge-tap carries 1 contamination error vs native's 4 starvation errors; the
   kind-weighted selection (`decomposition_search.rs:797-815`) exists precisely
   because that trade was judged wrong.
3. **Re-bless blast radius is corpus-wide**: all 8 stress goldens, all
   `StressBaseline` ceilings, candidate-trigger semantics (k1-shape-fix /
   ModuleSizeSplit lose their enrollment trigger), KC4 area-win delta tables
   unmeasured at flip scope, KC6 junction-regression gate, KC5 in-game anchor
   (open, user-run).

Current state: utility@10/s on main = 46 errors (4 dead-end + 8 item-isolation +
34 unresolved-junction). Merge-tap wins selection only on that cell (46 vs 175),
in the ignored `measure_utility_10s_am3`. No env var / UI knob exists to force
merge-tap (`wasm-bindings/src/lib.rs:77` hard-codes false). Followups docs carry
**zero** merge-tap items — the entire open worklist lives in the RFC decision log
(itself a small documentation debt for cold pick-up).

**Honest sequencing if funded:** (3,7) endpoint fix → merge-tap-aware lane
ordering (or accept 34-junction residue with KC6 re-scoped) → flip the two code
sites → corpus re-bless → KC4/KC6 delta tables → KC5 anchor (unblocked by §8.3).

### 8.2 Solver fidelity gaps

**GAP 1 — Base productivity on the None path. Verdict: needs RFC (small one, or
explicit amendment executing RFC-044's recorded followup).** Fix is ~5 lines in
`resolve_machine_modules` (hoist `base_effect` read above the `None` early-return,
`module_policy.rs:121-123`); the three netflow scaling sites are already
prod-generic. Blast radius is NOT the SA machine counts — it's **free-mode recipe
selection flips on Nauvis solves**: with ×1.5 on molten-iron and casting, every
from-ore solve flips plates from furnace smelting to foundry casting chains
(≈0.45 raw-input cost/plate vs 1.0). All 8 STRESSGOLD goldens re-bless; calcite
becomes a new default external input; foundry fluid rows enter default layouts.
Pre-scoped: recorded in `recipe_db.rs:100-106`, RFC-044 rev 2 non-goal
(`rfc-044-machine-modules.md:237-238`), close-out `:407-417`.

**GAP 2 — Unknown machines solve silently at 1.0. Verdict: small PR.** Split the
API: keep `get_crafting_speed` lenient (only `analysis.rs:408` needs it — imported
community blueprints with modded entities), check
`db().machines.contains_key(...)` at the two solver sites (`solver.rs:394`,
`netflow.rs:602`) and return `MissingCraftingSpeed`. Makes a dead error variant
live; bit-identity preserved for all known-machine solves; no golden movement.
Note: `recipe_db.rs:512-515` currently *asserts* the 1.0 default — update with
the split.

**GAP 3 — Machine selection in the LP. Verdict: needs RFC.** Mechanics are a day
(one `Column` per surviving (recipe, machine) pair in the finalization loop,
`netflow.rs:565-654`; capability preflight records error only when ALL variants
fail). The hard problems are economic: (a) the only machine-differentiating cost
is monotone in speed → LP always picks fastest; a real trade-off needs a
capital/footprint term → **frozen-cost-table revision (KC3-gated)**; (b)
split-machine solutions break one-spec-per-recipe assumptions in the parity
harness and tree-walk merge; (c) palette interplay — LP must not silently
override explicit user `craft=`/`smelt=` intent.

**GAP 4 — Strict available inputs. Verdict: small PR (opt-in flag).** No corpus
fixture relies on the soft behavior (checked: parity corpus + stress configs use
ores-only or plates-only; no test declares copper-cable/iron-stick available).
Add `strict_available_inputs: bool` to `NetflowOptions` (`netflow.rs:78-108`),
skip all-available-output columns (preserving byproduct co-producers like AOP),
thread through solver/wasm/sidebar following the quality/module plumbing
precedent. Default-off preserves bit-identity. Note: the legacy tree walk is
ALREADY strict (`solver.rs:366-369`) — strict mode brings free mode closer to the
compat oracle. Design call to pin: the co-product exclusion rule.

### 8.3 Headless Factorio verification (feasibility: CONFIRMED, 2–4 days)

**Verdict: feasible and cheaper than it looks.** Every mechanism exists in
shipping 2.x and has community precedent. Full research trail available; summary:

- **Headless Lua execution**: `--start-server-load-scenario` (scenario
  `control.lua` `on_init`), or `--benchmark save.zip --until-tick N` —
  self-terminating, no RCON, flat-out simulation. No player exists headless;
  scripts must avoid `game.players`.
- **Blueprint import via the game's own parser** (the anchor that matters):
  `LuaItemStack::import_stack` (return code itself is a validation signal) →
  `LuaItemCommon::build_blueprint{build_mode=superforced}` → returns ghosts →
  `LuaEntity::revive()` each. Module requests survive as `item-request-proxy`
  entities — must insert modules and destroy proxies explicitly.
- **No credentials needed**: `factorio.com/get-download/latest/headless/linux64`
  anonymously 302→200 (verified). factoriotools/factorio docker image (10M+
  pulls) downloads at build time, SHA256-pinned, has `DLC_SPACE_AGE` toggle.
  **First-hour smoke test**: confirm Space Age prototypes load on anonymous
  headless (community evidence says yes; not a Wube statement). Don't publish
  baked images publicly (EULA).
- **Existing tooling**: GlassBricks/FactorioTest (actively maintained busted-style
  framework with an npm CLI that launches Factorio headlessly and collects
  results — the orchestration problem solved). No community tool does end-to-end
  import→build→measure; the harness itself would be new code.
- **Harness sketch**: `research_all_technologies()` (mandatory: stack inserters,
  belt stacking); spawn `electric-energy-interface` per pole-network component
  (blueprints have no power generation); feed boundary inputs per the solver's
  external-input manifest (infinity-chest + `loader-1x1` per boundary lane;
  `infinity-pipe` for fluids); drain outputs (loader → infinity-chest remove
  mode) so backpressure can't falsify the measurement; warmup ~2–7k ticks +
  steady-state detection; read `LuaForce::get_item_production_statistics(surface)
  .get_flow_count{category="input"}` (NOTE: "input" == produced — inverted vs
  intuition); `helpers.write_file("result.json", ...)`.
- **Cost**: ~25–45 s wall per blueprint at USP scale (6.6k entities, 200–400 UPS
  conservative) + ~10 s startup. ≤1 min each, embarrassingly parallel. A
  10-blueprint CI gate ≈ 1–2 min on 4 cores.
- **Build order**: tier 1 (gear) → tier 2 (EC) → tier 3 (fluid) →
  `science_gauntlet`. First green USP@1/s retires every open in-game anchor
  (inserter-sizing KC5, build-quality, merge-tap KC5) in one stroke.
- **Risks**: Space Age entitlement (smoke-test first); module proxies (handle
  explicitly); measurement backpressure/warmup (drain + steady-state detection);
  quality flow statistics need per-quality `FlowStatisticsID`; Gleba spoilage
  needs net-fresh-rate measurement if ever generated.

---

## 9. Suggested sequencing

| Order | Item | Size | Unblocks |
|---|---|---|---|
| 1 | Doc-rot fixes A–J, L–Q (§7) | docs-only PR | — |
| 2 | Dead code removal K + O | small PR + test run | — |
| 3 | GAP 2 (unknown-machine hard error) | small PR | data-pipeline trust |
| 4 | GAP 4 (strict available inputs, opt-in) | small PR | UI semantics honesty |
| 5 | Headless harness spike (§8.3) | 2–4 days | ALL open in-game KCs; converts the whole confidence chain |
| 6 | End-to-end rate audit check + port-fluid identity + wrong-item-feed | small-medium | catches what #5 can't run in unit tests |
| 7 | GAP 1 RFC (base productivity None path) | small RFC, corpus re-bless | SA solve correctness |
| 8 | Templates-return-geometry contract | medium refactor | deletes mirrored-constants bug class |
| 9 | Merge-tap Phase 2 funding decision (§8.1) | medium-large phase | balancer crisis, #135/#136, utility@10/s |
| 10 | Beacons (RFC) | medium-large | endgame fidelity + layout simplification |
| 11 | Max-flow audit walker | medium | honest rate upper bounds, min-cut diagnostics |
| 12 | GAP 3 RFC (LP machine selection) | RFC + KC3 cost-table work | solver optimality |
