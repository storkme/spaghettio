# Project status ledger

**Status (2026-07-21)**: moved out of `CLAUDE.md` so the agent-context file
sticks to process; this is the canonical home for capability status now.
Update this file (not `CLAUDE.md`) when a tier's status changes or an RFC
closes out. Per-topic backlogs stay in their own `*-followups.md` docs; this
ledger is the cross-cutting view.

Fully re-audited 2026-07-21 (fresh `science_gauntlet` run + default-suite
sweep + issue-state check); per-row history trimmed to current status —
the evidence trails live in the owning RFC decision logs.

## Measurement protocol

Layout results are only comparable with the SAT zone cache pinned:

```bash
SPAGHETTIO_ZONE_CACHE_PATH=$(pwd)/crates/core/data/sat-zones-ci.bin \
    cargo test --manifest-path crates/core/Cargo.toml
```

This is what CI does (see the comment in `.github/workflows/ci.yml`). Without
the pin, a fresh environment solves zones live under wall-clock budgets, and
slow/loaded machines record spurious timeouts that then *cache*, producing
deterministic-looking `unresolved-junction` errors that reproduce nothing
about the code. (Verified 2026-07-21: an unpinned fresh container fails
`tier4_advanced_circuit_from_ore_am2` and shows production/utility gauntlet
FAILs; the same commit with the pin is green everywhere.) The gauntlet run
writes newly-solved zones back to the pinned path — `git checkout` the pin
file afterwards unless you're deliberately re-blessing it.

## Recipe complexity ladder

Tracks which recipes produce zero-error bus blueprints. Moving up = real
progress. Tests for each tier live in `crates/core/tests/e2e.rs`; all rows
below are gated by the default (non-ignored) suite unless noted.

| Tier | Recipe | Complexity | Bus status |
|------|--------|-----------|-----|
| 1 | `iron-gear-wheel` | 1 recipe, 1 solid input | SOLVED — clean, incl. 20/s |
| 2 | `electronic-circuit` | 2 recipes, 2 solid inputs | SOLVED — clean from ores; stress-gated at 20/22/23/30/35/40/s (yellow) and 60/s (red) from ore |
| 3 | `plastic-bar` | 1 recipe, 1 fluid + 1 solid input | SOLVED — clean, incl. from crude; sulfuric-acid, heavy-oil cracking, and multi-machine advanced-oil-processing also gated at this tier |
| 4 | `advanced-circuit` | 5+ recipes, mixed solid/fluid | SOLVED — from plates fully clean; from ore (AM2) green with 1 known input-rate-delivery warning (pre-existing demand-pull modeling residual). Partitioned 4/s + 5/s and horizontal-stack 7/s stress-gated. |
| 5 | `processing-unit` | Deep chain, multiple fluids | SOLVED — from ore (AM3, 2/s) fully clean; horizontal-stack gated at 2/s (pipe bypass) and 25/s (pole coverage). Higher-rate partitioned strategies still have junction + starvation issues — `partition_strategy_scoreboard_extended`. |
| 6 | `flying-robot-frame` | Adds lubricant: advanced-oil-processing refinery rows with 3 fluid outputs | SOLVED via the USP chain (0 errors). No dedicated FRF fixture yet. |
| 7 | `utility-science-pack` | Very deep chain (LDS + PU + FRF) | SOLVED — fully clean at 1/s (gauntlet 2026-07-21: 0 errors, 0 warnings, 6796 entities, 208×285). |

### Six-pack scoreboard (gauntlet run 2026-07-21, 1/s, CI-pinned cache; re-verified bit-identical 2026-07-22 on the post-RFC-044..047 tree)

| Pack | Size | Entities | Result |
|------|------|----------|--------|
| automation | 39×36 | 281 | PASS |
| logistic | 41×79 | 634 | PASS |
| military | 45×91 | 1002 | PASS |
| chemical | 83×137 | 2392 | PASS |
| production | 227×156 | 4115 | WARN — 8 inserter-item-throughput (6 input3 contest-losses + 2 far-side rate walls; [`inserter-throughput-followups.md`](inserter-throughput-followups.md)) |
| utility | 208×285 | 6796 | PASS |

The only residual across all six packs is production-science's 8
inserter-item-throughput warnings.

### Beyond the ladder — capabilities the default suite also gates

The tier table understates current capability; these are all regression-gated
on every push:

- **Self-loop / byproduct chains** (net-flow solver): Kovarex enrichment,
  uranium processing (surplus export + voider variants, voider purity),
  pentapod-egg, fish-breeding, and bacteria self-loops.
- **Space Age machines**: electromagnetic plant (superconductor), cryogenic
  plant (fusion power cell), foundry (molten iron), biochamber
  (biolubricant); substation as a first-class entity.
- **Fulgora**: scrap-sorting mechanism (multi-output recipe handling).
- **Build quality** (normal→legendary): quality-aware machine counts,
  inserter ladder, pole supply/wire reach; differential fixtures pin
  Normal bit-equality; EC@45/s express-legendary-from-ore green with the
  1 known input-rate-delivery residual.
- **Machine modules** (RFC-044): global speed/productivity policy →
  per-machine loadouts, effect-scaled machine counts, 2.0 insert-plan
  export, slot/eligibility validators, web slot overlay. In-game paste
  anchor CLOSED (user-verified, four inventory classes).
- **Belt stacking** (RFC-046, S∈{1..4}) and **lane-aware delivery**
  (RFC-047): rate ceilings scale ×S; the EC@60/s red-from-ore config is
  physically valid end-to-end at S=2 (in-fixture per-tile capacity audit),
  and the legendary-express@60 headline is gated
  (`stacking_ec_60s_express_legendary_s2`).
- **Rate headroom caveat (S=1 only)**: *unstacked* final-product output
  above one belt's capacity is still over-committed onto a single merger
  belt and the lane-throughput check doesn't visit merger tiles
  ([#311](https://github.com/storkme/spaghettio/issues/311)) — treat
  unstacked >45/s "clean" results as routing-verified but not
  throughput-verified until #311 closes.

### Scaling walls (scaling gauntlet run 2026-07-22 post-RFC-047, release, 180s/cell budget)

`science_scaling_gauntlet` result matrix (rows = pack, columns = rate):

| Pack | 1/s | 2/s | 5/s | 10/s |
|------|-----|-----|-----|------|
| automation | PASS | PASS | PASS | PASS |
| logistic | PASS | PASS | WARN×3 | WARN×7 |
| military | PASS | PASS | LAYOUT-ERR | PASS |
| chemical | PASS | PASS | PASS | FAIL×4 |
| production | WARN×8 | WARN×14 | FAIL×4 | LAYOUT-ERR |
| utility | PASS | FAIL×2 | WARN×36 | TIMEOUT |

First walls: logistic 5/s (inserter-item-throughput×3), military 5/s
(honest refusal: RFC-047's late sideload check names a stone-brick
25/s-over-22.5/s sideload-fed single trunk; the (n,1) merge-tap fallback
that would fix it is [#336](https://github.com/storkme/spaghettio/issues/336)
— note 10/s passes, the wall is shape-specific, not monotone), chemical
10/s (belt-loop, underground-belt, unresolved-junction×2), production 1/s
(the known 8), utility 2/s (belt-loop, underground-belt,
input-rate-delivery×5). Automation passes through 10/s.

Drift vs the pre-RFC-047 run (2026-07-21): military 5/s PASS →
LAYOUT-ERR and production 10/s TIMEOUT → LAYOUT-ERR are the new checks
converting silently-broken/timeout outcomes into named refusals;
chemical 10/s improved FAIL×6 → FAIL×4 (its two lane-throughput errors
died with the RFC-047 row consolidation).

Caveat: cells beyond the CI pin's zone coverage solve live under
wall-clock budgets, so the TIMEOUT and unresolved-junction counts are
machine-dependent (measured on a remote container). The belt-loop and
underground-belt *errors* are genuine layout defects independent of solve
budgets — utility@2/s FAIL×2 is the most reachable new fix target.

## Recent RFC close-outs

**`rfc-057-topology-preserving-dense-repacking.md` snake fold (2026-07-29,
PR #481 — RFC ACTIVE, not closed)**: a single fold is Factorio-verified.
`chain-mil5ore` goes 552x32 (17.25:1) to 276x66 (4.18:1) at **5.00/s produced
with 146 of 146 machines working** — the unfolded control's exact census, both
runs converged at `--warmup 216000`. Bus layouts fold too: `gear15-ore` reaches
55x65 (1.18:1) on **three** folds. Folding stays refused as a *density* lever
(measured routing headroom ~20%); its value is shape, since a 2,381-tile-wide
ribbon is not a factory anyone builds. Reachable only from tests — deliberately
not wired to a `LayoutOptions` flag yet.

Carries a **validator finding that outlives the fold**: an earlier version
validated at exact control parity and produced 0.00/s in Factorio, because a
relocated output belt left its `boundary_outputs` record behind. Geometry-only
validation cannot certify a transform that moves a boundary. Fixed separately in
PR #482. Open blockers and measured refusal causes:
[`snake-fold-followups.md`](snake-fold-followups.md).

**`rfc-053-direct-insertion-cells.md` Phase 0 (2026-07-25, PR #436 —
RFC ACTIVE, not closed)**: machine→inserter→machine DI, the topology
#429 asked for and the corpus overwhelmingly builds. Evidence is now
reproducible from a clean checkout via the tracked `di-patterns`
miner (`cargo run --release -p spaghettio_mining --bin di-patterns`):
16,507 DI observations, 4,116 of them `copper-cable →
electronic-circuit`, of whose top-20 geometry patterns (3,866) all but
one use a **1-tile gap**. **KC1 (ratio feasibility) PASSED with
margin** — the worst case across the corpus top-10 is the canonical
cable→EC at 2.50/s per inserter slot against 19.2/s available from a
stack inserter at the L2 default; the feasibility rule reduces to
`machine_feed_rate ≥ 2.5/s`, satisfied at engine defaults and for a
`Fast`-capped user. **KC6 (fluid coverage) FIRED (5/10 vs a threshold
of 2)** and was diagnosed as a criterion-specification defect —
it conflated a *fluid coupling* (impossible: inserters cannot move
fluid) with a *fluid-adjacent machine* (common), and counted pairs
unweighted where its rationale was demand (solids-only actually covers
**69.4%** of top-10 instances, and the dominant pair is fully solid).
Resolution was the criterion's own prescribed action — **re-scope, not
reprieve**: pipes moved out of Non-goals into required Phase 2 scope.
Recorded data gap: `electric-furnace → electric-furnace` is the
2nd-commonest DI pair (1,585) but is invisible to recipe-keyed analysis
— furnaces carry no explicit recipe.

**`rfc-053` Phase 1 COMPLETE (2026-07-25, PR #452 — RFC still ACTIVE,
Phases 2–4 remain)**: `place_rows` fuses an eligible producer/consumer
pair into ONE cell row, so the engine emits true
machine→inserter→machine DI for the first time. **Inert by default**
(`direct_insertion: false`) — no existing layout moved. Three more kill
criteria evaluated, all passing: **KC3 (honest throughput)** —
sim-measured 2.24/s delivered against 2.00/s planned (112%), 0
validation issues, 32/32 machines working, *with a DI-off control on the
same target* that delivers the identical 2.24/s, attributing the +12%
overshoot to a solver rate-model artifact rather than to DI; **KC4
(density)** re-confirmed end-to-end at 213 entities vs the bus control's
335 (−36%); **KC2 (face contention)** — passes at L2 but with **zero
margin**, 1 near + 2 far = 3 of 3 columns, which constrains Phase 2's
design. Coverage measured rather than assumed: 4 of 11 real targets
build cells, every refusal with a named cause, and the ceiling is a
**fan-in belt limit** (one belt cannot feed a high-rate cell), not a DI
limit.

**`rfc-053` Phases 2 + 4 landed (2026-07-25, PR #459)**: the
horizontal ROW cell — producers and consumers interleaved in one row,
coupled east/west in the 1-tile gaps — chosen on corpus evidence
(`di-patterns faces`: the dominant real shape is `DI@E+W | S:in1 S:out1`,
both remaining flows on one face at reach-1, opposite face free). It
needs **no reach-2 inserter and no research** (stack moves 12.0/s at L0
against a 5.0/s requirement) and reuses `place_rows` rather than
replacing it. **Both TOP corpus DI pairs now build, validate at 0 issues
and sim at/above plan**: `copper-cable → electronic-circuit` (#1, 4,116
instances) 101.3% delivered, 50/50 machines working;
`electric-furnace → electric-furnace` (#2, 1,585) 109.5%, 32/32. Phase 4
threads `direct_insertion` through wasm, the worker, URL state (`di=1`)
and a sidebar checkbox — **still off by default**, since a pair the
engine cannot serve as a cell falls back to the bridge and then the bus.
**Fluid-fed PRODUCERS now ship too (2026-07-25, same PR)**: both
`casting-*` → EC pairs (#3 at 544 instances and #4 at 339) build a row
cell, validate **0 errors 0 warnings** across 2.5–20/s (both channels —
`validate()` AND `LayoutResult.warnings`; the latter carried a false-alarm
fluid-branch warning until 2026-07-26), and sim at
**101.3% delivered / 100.0% produced**. They needed three things, each
found only by attempting an end-to-end build: the pipe cut, heterogeneous
footprints (5×5 foundry beside a 3×3 assembler, bottom-aligned), and a
`belt-connectivity` exemption — a piped producer hands its product
straight to its neighbour, so no inserter of its ever touches a belt.
Without the cell neither pair lays out at all today, so this is the only
path by which a fluid-fed producer works. A fourth prerequisite ("ratio
tolerance") was **claimed and then disproved**: it came from feeding
`plan_row_straddle` raw per-machine rates, where the caller passes
utilization-scaled ones.

The fluid-drawing CONSUMER shape is **built but unreachable**, and the
sim is the only reason we know:
`solid-fuel-from-light-oil → rocket-fuel` (652 instances) produced a
cell validating 0/0 that made **literally nothing** — the solver
resolves `rocket-fuel` to a burner `biochamber` and nothing in the
engine delivers burner fuel. `cell_machines_are_powerable` now refuses
non-electric roles; the engine-wide gap is **issue #461**.

**A FIFTH corpus pair (2026-07-26, PR #470)**: slot-based straddle
emission. `plan_row_straddle` could only place a consumer on a producer's
**right**, but every producer has two neighbours, so the gap between
`P_i` and `P_{i+1}` holds up to two consumers. `copper-cable →
space-platform-foundation` (353 instances) balances *exactly* — 4
producers at 5.0/s against 8 consumers at 2.5/s — and a valid
arrangement always existed (`C0 P0 C1 C2 P1 C3 …`); the append-only walk
simply could not express it, then the adjacency invariant correctly
refused what it had built. **The geometry was feasible the whole time.**
Each producer now gets an explicit left and right slot, a
doubly-fed consumer taking the whole gap. SPF goes `cell=0` + 4 warnings
→ `cell=181`, **0 errors 0 warnings**, and sims **PASS at 98.7%
delivered, 24/24 machines working**. Measured against `origin/main` by
restoring the old file, since a green suite cannot distinguish a
regression from a pre-existing refusal; the three sim-verified pairs
above are byte-identical after.

The next blocker is **face allocation, not straddle**: consumers with
three or more solid inputs. Measured by probing solves rather than
inferred from recipes — **351 instances, not the 1,029 first claimed**
(`iron-stick → rail` alone; the other two candidate pairs are blocked
upstream for unrelated reasons). Two designs were reviewed before
building either: moving a row's output belt north is **rejected** —
`output_merger` assumes a south-facing chain universally, so it is a
rework, not a stamping change — while adding the second consumer input
on the **north** face survives review and is recorded in the RFC as a
candidate with one named prerequisite (`row_cell_eligible`'s same-item
guard uses `.find()`, which checks only one of the three pairs). **Not
built; the decision is deliberately open.**

Open against this RFC: modules refuse (the module post-pass keys
`(entity, recipe)` off `row_spans` and a fused row contributes only the
consumer's recipe); KC5 (solver escalation bound) is still unevaluated;
and the row cell's rate ceiling is bounded by ratio **alignment** — at
P30:C23 the flow intervals put three producers against one consumer,
while P20:C15 (exactly 4:3) is fine at any scale.

Corpus `fan` analysis redirected Phase 3: fan-in >2 is only 2.1%
of the corpus and neither dominant shape uses stacked bands, so
multi-band is a small tail and **Phase 2 (face allocation) should come
first**. Open tracking items: the ~10% electric-furnace steel rate
under-prediction the KC3 control exposed (orthogonal to this RFC), and
`ci.yml`'s `pull_request: branches: [main]` base filter, which silently
gives any **stacked PR zero CI**.

**`rfc-052-oil-mega-cell.md` close-out (2026-07-24, Phases A/B/C —
PRs #401/#403/#405/#408/#411/#421)**: fluid subgraphs compose as
UNCROPPED mega-cells inside solid chains. Delivered: the first
working refineries in project history (#400's three stacked defects),
the game-faithful pipe-UG reach model (#407: entity-distance 10, not
gap 10 — sim-falsified), sim-measured fluid PORT IDENTITY (mirrored
refineries bind fluids x-DESCENDING; the old ascending zip starved
them in-game — FFF #394 class, pin-tested), chain-fed mega inputs,
multi-consumer export fans, and three latent engine-bug classes fixed
(hop pair-destroyer ×2, tapoff release/retain hole). Bus-refusal wins
gated at chem-pack@5, PU@4, USP@2 (each composes 0 errors where the
bus hard-fails; USP@2 = 48k entities, the largest layout produced).
Sim evidence (re-measured 2026-07-24 at the post-#431 L2 default):
plastic/sulfur/AC-from-raw PASS and registered; **chem-pack@5 now
PASSES at plan (5.00/5.00 exact, 172/172 working) and is REGISTERED**
— it never carried the #383 deficit class. **PU@4 FAILS at −27.3%**
and stays unregistered ([#437](https://github.com/storkme/spaghettio/issues/437));
its original inserter attribution was disproven (the warnings are a
validator utilization artifact on composed layouts — each of K=8
replicas is charged the whole chain's demand), so the deficit is real
but unattributed. **USP@2 measured 2026-07-25: −57.2% FAIL**, converged
and properly sampled on the post-#464 instrument (5 checkpoints, full
300-item windows auto-sized to 21,240 ticks, group drift +0.4%);
unregistered, tracked at
[#453](https://github.com/storkme/spaghettio/issues/453). Re-measured
at the **blessed L0 geometry** after [#466](https://github.com/storkme/spaghettio/pull/466)
fixed the fixture exporter: −57.3% FAIL, converged over 9 windows flat
across 47 game-minutes (mean 0.850/s, spread 2.9%, net trend −0.35%).
Geometry made no difference — the pre-#466 L2-geometry fixture measured
−57.2% in a matching L2 world, so that run was self-consistent rather
than mixed-world, and both configurations perform identically. Tracked
follow-ups: #383 (bound lift), #409 (landed independently), #423
(pitch-1 splitter-passthrough). Full trail:
[`rfc-052-oil-mega-cell.md`](rfc-052-oil-mega-cell.md) decision log.

> **Instrument caveat on the numbers above** ([#454](https://github.com/storkme/spaghettio/issues/454)
> / [#464](https://github.com/storkme/spaghettio/pull/464), 2026-07-25).
> Every mega-cell rate measured before 2026-07-25 came from a harness
> whose stability test **certified decelerating ramps as converged** —
> it compared only the last two windows, which any flattening ramp
> eventually passes, at a point short of its asymptote. chem5's blessed
> "5.00/s EXACT" was such a point (its series: 4.62 → 4.92 → 5.00, span
> mean 4.84). Re-measurement under the group rule holds the verdict
> (5.08 produced / 5.15 delivered, **PASS**, within `check` tolerance of
> the blessed entry), so chem5's registration stands. **PU@4's −27.3% has
> NOT been re-measured** and should not be treated as a settled number
> until it is. USP@2's deficit reproduced across all three instrument
> versions and at both geometries (−57.0 / −57.4 / −57.2 / −57.3), the
> last of those on the post-#466 blessed fixture with 9 flat windows, so
> the number is settled. An earlier reading that it was "still climbing"
> came from a 3-window probe that itself reported NOT CONVERGED; it did
> not reproduce and has been withdrawn.

**`rfc-inserter-sizing.md` close-out (2026-07-13)**: bus inserters sized to
planned per-machine throughput via a shared regular→fast→stack ladder
(long-handed count-ladder for reach-2 sides), with an ingredient-to-belt
reassignment lever and a user-facing `max_inserter_tier` engine param
(wasm-bindings + web UI, URL-encoded). Six-pack warning trail: 140 → 12 at
close-out → 8 after the last-in-row belt extension (`0d7132c`); the
remaining 8 are production-science, root-caused 2026-07-20 (6 input3
contest-losses + 2 genuine far-side rate walls — see
[`inserter-throughput-followups.md`](inserter-throughput-followups.md)).
Validator-verified only — the RFC's two in-game blueprint-import anchors
(kill criterion 5) remain open until the user runs them; full trail in
[`rfc-inserter-sizing.md`](rfc-inserter-sizing.md)'s decision log.

**`rfc-build-quality.md` close-out (2026-07-20)**: user-facing **build
quality** param (normal→legendary, `quality`/`q=` URL-encoded through wasm
`solve`+`layout` and the sidebar). Solver machine counts scale
×(1+0.3·level) via `effective_crafting_speed`; the inserter ladder, pole
supply radii, and wire reach are quality-aware; functional entities get
`PlacedEntity.quality` stamped; validators rate each entity by its own
tier; export emits (and the parser reads) the lua-api `quality` field.
Normal is bit-identical to pre-RFC. The 60 EC/s legendary headline stays
capped at 45/s until [#311](https://github.com/storkme/spaghettio/issues/311)
closes; [#312](https://github.com/storkme/spaghettio/issues/312) tracks the
quality-magnified consumer-clamped fan-in wall. **In-game import anchor
still open** (user-run). Full trail: [`rfc-build-quality.md`](rfc-build-quality.md)
decision log; renderer constraints learned en route: `web/CLAUDE.md`.

**`rfc-043-pole-band-thinning.md` close-out (2026-07-20)**: quality-aware
pole-band thinning landed (Phase 1; Phase 2 cross-row sharing deferred) —
closed [#310](https://github.com/storkme/spaghettio/issues/310) via PR #318.
Registry: [`rfcs.md`](rfcs.md).

**`rfc-044-machine-modules.md` close-out (2026-07-21, all 4 phases)**:
user-facing **module policy** param (speed/productivity × tier 1–3 ×
optional module quality, compact `m=`/`modules=` URL form e.g. `p3l`,
through wasm solve/layout and the sidebar). `module_policy.rs` is the
single source: one global policy resolves to per-machine loadouts and
effect factors; ineligible (machine, recipe) pairs get NO modules (prod
falls back to empty, not speed); netflow machine counts scale by the
1%-floored effect formula. Export emits the Factorio 2.0 insert-plan
encoding with the per-class inventory table (parser reads it back).
Validator checks 24–25 (`module-slots`, `module-eligibility`) are
WARNING severity by design — invalid loadouts don't fail a paste;
eligibility gates on the module's *beneficial* effect only (the
"effects ⊆ allowed_effects" rule was falsified by draftsman data). Web
slot overlay ships alongside. Corpus evidence: 198/198 community files
sweep with zero module warnings. **In-game anchor CLOSED** (KC2:
user-pasted four-inventory-class anchor verified in Space Age) — the
only recent arc with its game anchor closed. Full trail:
[`rfc-044-machine-modules.md`](rfc-044-machine-modules.md) decision log
(PRs #321/#322/#323/#325).

**`rfc-046-belt-stacking.md` close-out (2026-07-21)**: user-facing **belt
stacking** param (off/×2/×3/×4 = Space Age belt stack size research,
`stacking`/`st=` URL-encoded through wasm `layout*` and the sidebar; solver
untouched). Belt tier selection, lane caps, merger capacity, and the
validators scale ×S via `common::*_stacked` helpers; belt-dropping output
sides are forced to stack inserters at S>1 (`size_belt_drop_side`); a
static family-level exemption (`bus/stacking_ctx.rs`) keeps uniform ×S
sound for unstackable producers (self-loop/kovarex, D2b secondary outputs,
recycler ejection — validators re-derive it independently, per-tile).
Full-belt delivery thresholds initially did NOT scale — *superseded by
RFC-047 (2026-07-22), which grounded and scaled them.* Headline: the #311
stress config (EC@60/s red from ore) is **physically valid end-to-end at
S=2**, proven by an in-fixture per-tile capacity audit. S=1 is
bit-identical to pre-RFC (zero golden re-blesses). Mechanics:
`factorio-mechanics.md` BS1–BS7. In-game import anchor open (user-run;
[#335](https://github.com/storkme/spaghettio/issues/335)'s one-bank
ore-routing warnings persist on legendary-express). Full trail:
[`rfc-046-belt-stacking.md`](rfc-046-belt-stacking.md) decision log.

**`rfc-051-cell-composition-integration.md` close-out (2026-07-22,
updated 2026-07-23)**: cell composition is a **production path ON BY
DEFAULT** — `CellComposedCandidate` (default Candidate since the flip;
`cc=off` escape hatch) competes unbiased in the decomposition search
for solid tree-with-fan-out chains; strictly additive (bus wins on
area where both succeed, composition surfaces only on refusals; suite
+ goldens unchanged under the flipped default). The chain auto-placer:
engine-generated cells, two-registry crossing Router, merge cascades,
fan-out trees, south bypass, and **ratio quantization** (K side-by-side
copies at 1/K rate so no corridor/feed exceeds express 45/s; K=1
bit-identical, proven by the registered geometry hash). Composed
coverage at the validator level: EC@15 (the
[#336](https://github.com/storkme/spaghettio/issues/336) refusal),
EC@15-from-ore (furnace cells), EC@30 and EC@60 (pre-quantization
refusals; bus validation-fails ec60), and **mil5-from-ore** (9 specs,
K=2, 0 errors — the military 5/s scaling wall's #336-class refusal;
PR #393 fixed the Router's boundary-blind hops and added westward
bypass support after the placement order proved consumers can sit
west of producers). mil5-from-plates' composed candidate is 0/0 but
the search still returns the broken native layout there
([#392](https://github.com/storkme/spaghettio/issues/392) — resolved:
validation-tiered selection). Measured-at-plan claims live in the
sim-verified registry (geometry-hashed AND world-keyed per #391 —
declared capacity/stacking are checked fields): currently
AC-from-plates (PASS −0.3%) and mil5-from-plates (PASS, delivered
5.00/s exact — first physical validation of the westward bypasses),
both at declared capacity 0 in the post-#390 honest world. EC-row
geometries are RE-ATTRIBUTED but still WARN (revises the 2026-07-24
policy on
[#383](https://github.com/storkme/spaghettio/issues/383)). The output
half of the old attribution is falsified:
[#431](https://github.com/storkme/spaghettio/issues/431)'s declared-
level sweep ran the same byte-identical bridged EC row at L0..L7 and
measured it delivering the FULL 15.00/s at L2+ with zero output-blocked
machines at every level, so the belt was never the bind; [PR
#434](https://github.com/storkme/spaghettio/pull/434) recalibrated
`ROW_LANE_FACTOR_BRIDGED` 1.733 → 2.0 accordingly and those warnings no
longer fire. But the composed chain does NOT reach plan. Measured
2026-07-24 at the L2 default: chain-ec15 14.10/15.00 (−6.0%),
chain-ec30 28.40/30.00 (−5.3%), and the declared sweep PLATEAUS
(d1 −8.0% → d2 −6.0% → d7 −5.3%) with an identical census at every
level (1 output-blocked, 1 ingredient-starved, 13 working). So the
deficit is two things: a research-bound input component that L2 clears,
plus a **level-invariant ~5.3% structural residual that is neither
research-reachable nor the row's output belt — currently unattributed
and, since the recalibration, unwarned** ([#435](https://github.com/storkme/spaghettio/issues/435)). Entries stay WARN with the
corrected attribution; they do not graduate to at-plan. ~~mil5-from-ore
FAILs flat at −28.7% (firearm rows' inserter COUNT) and stays
unregistered.~~ **RETRACTED 2026-07-26** — both the number and its stated
cause. The same fixture measures **+0.7%, 146/146 machines working, PASS**
at `--warmup 288000`; the −28.7% was a buffer-fill transient, so there was
no inserter-count defect to attribute. See §"Default warmup is too short
for deep chains" below. Registration still pending a corpus-wide
re-measure, which must be justified by the oracle alone. Full trail:
[`rfc-051-cell-composition-integration.md`](rfc-051-cell-composition-integration.md).

**`rfc-048-cell-composition.md` Phase-1 close-out (2026-07-22, PR #365)**:
the cell-composition method delivered its existence proof — a composed
EC@15/s-from-plates factory (engine-generated cells, segment-crop
extraction, contract-ported corridors) **runs at plan in headless
Factorio**: 15/15 machines working, produced 15.00/s, converged — on
the exact config the bus engine refuses
([#336](https://github.com/storkme/spaghettio/issues/336)). All five
kill criteria PASS (kill 3 over its 2× area boundary at 2.48×, spared
by the criterion's compensating-win clause: the engine has no layout
here at all). Permanent gates: `cell_composed_ec15_zero_errors` (0
errors, warnings pinned ≤6 sim-adjudicated) and
`cell_composed_plastic_zero_issues` (fluid-consumer composition, 0/0).
Fluid sim verification blocked harness-side
([#364](https://github.com/storkme/spaghettio/issues/364) — the
infinity-pipe feed path delivers nothing for ANY layout, proven by
controlled attribution); sim-kit composition rules learned en route
live in [#363](https://github.com/storkme/spaghettio/issues/363).
Verdict: **GO for the Phase-2 integration RFC.** Full trail:
[`rfc-048-cell-composition.md`](rfc-048-cell-composition.md) decision
log + Phase-1 close-out section.

**`rfc-049-inserter-capacity-research.md` close-out (2026-07-22)**: user-facing **inserter capacity research** param (level 0–7, `inserter_capacity`/`ir=` URL-encoded, sidebar "Inserter research"). Schedule pinned from raw wikitext with 2-fetch reproducibility (bulk 2→12; stack = bulk+4 → 6→16; non-bulk 1→3 via the chain +1 from Transport-belt-capacity-2 → 4) — summarized wiki fetches are BANNED as constant sources (two contradicted each other; the failure mode reproduced live in review). Output belt-drop sides originally scaled linearly (swings × researched hand, with BS3 rounding — healing is exactly `hand ≡ 0 mod S`, non-monotonic: I8b) — **superseded 2026-07-23 (#385)**, see below. Input (belt-pickup) sides stayed at the L0 floor pending measured data — **closed 2026-07-22 (Phase 2, PR #378, #343)**: a 25-cell sim calibration matrix (tech-state-parity harness, all 8 levels for stack/bulk) measured belt→machine intake; `common::machine_feed_rate` now credits hand-ratio rates for non-bulk/bulk (measured conservative, 1.04–2.27× margins) and a measured floor table for stack (its real curve is non-monotone in hand size — dips at hands 7/14 — caught by the #376 adversarial review and confirmed by measurement). L0 bit-identical. **Phase 3 (2026-07-24, PR #381): the ladder now sizes to the declared level** — `size_side`/`contest_favors_far`/`capped_limit` take the research level at the measured `machine_feed_rate`; decided user+session 2026-07-22 (the axis is user-declared like belt tier). Gate closed honest-or-at-plan: the L7 fixture generates warned (`row-output-lane-budget`, plate row 15 vs 13 measured) and the sim confirms the priced floor. In-game anchor open (user-run; a legendary S=4/L7 export validates RFC-046/047/049 in one import). Full trail: `docs/rfc-049-inserter-capacity-research.md` decision log.

**#385 belt-drop min-form (2026-07-23)**: the RFC-049 belt-drop swing term (`swings × researched hand`) was never checked against the belt's own physical throughput — sim-measured onto yellow (true-S1 world) found stack credited 2–5× over the real 6.50/6.50/7.10/s (L0/L2/L7) and fast over-credited 44% at L7 (9.24 vs measured 6.40). `common::belt_drop_rate(name, quality, stacking, level, target_belt)` gained a `target_belt` param and now returns `min(swing_term, 0.85 × lane_capacity_stacked(target_belt, stacking))` — a stack inserter's flat 12.0/s credit onto a plain yellow belt (S=1, L=0) now credits 6.375/s, and non-bulk's L7 multiplier is sim-corrected 4.0→2.67. This deliberately breaks RFC-049's own L0-identity baseline for belt-drop (the 12.0 credit was never physically real) and RFC-046/049's "no recalibration" pattern for the output side specifically — the same "measured, never derived" discipline kill 2 already required for the input side. Threaded through the ladder (`size_belt_drop_side`/`size_side_output`, which lost their now-incorrect `stacking≤1 && level==0` shortcut) and the validator (`belt_drop_throughput`, which derives the drop tile's belt tier from the layout, falling back to yellow when none is found). One e2e fixture's expected inserter count changed (2→3 stack inserters, `fluid_multi_input_sulfur_output_uses_extra_column`); two constants-identity assertions updated (L7 non-bulk ×4.0→×2.67). Full suite clean (lib 798 / e2e 60 / netflow parity 10), clippy `--lib` clean, WASM rebuild clean, zero golden re-blesses. Full trail: `docs/rfc-049-inserter-capacity-research.md` decision log (2026-07-23 entry) and `docs/sim-harness-forensics.md`.

**#385 second half — row-output lane budget (2026-07-23)**: closed the
residual noted just above (the `[#385](https://github.com/storkme/spaghettio/issues/385)
output-side belt-drop class`) with a new check,
`validate::inserters::check_row_output_lane_budget` — a row's PLANNED
output (recipe demand × its share of the recipe's physical machines,
attributed via each machine's own output-inserter drop tile) compared
against what its belt-out can physically realize:
`LANE_UTILIZATION × lane_capacity_stacked(tier, stacking) × lanes_loaded`,
`lanes_loaded` 2 only with a genuine midpoint sideload bridge (tile-
adjacency detected — bridge and main line are edge-adjacent, unrelated
rows never are). Fires on `electronic-circuit@10/s`'s copper-plate row
(needs 15.0/s, a bridged yellow belt-out realizes 12.75/s) — the
sim-measured 7.4/s-per-lane gap now has a validator voice instead of
silently under-delivering (this fixture's in-game delivery has been
measured short of plan). Confirmed clean on `iron-gear-wheel@10`,
all three from-ore science packs. 6 e2e fixtures + 1 cell-composition
fixture gained the same structural warning (documented inline, not
tuned away); one cell-composition config
(`cell_composed_ec15_zero_errors`) briefly false-positived when the
check merged 3 independently sim-verified cells sharing one segment
string — fixed by switching row identification to tile-adjacency
clustering (pipeline-independent) rather than `LayoutResult::
effective_rows`, which the cell-composition pipeline never populates.
Full trail: `docs/rfc-047-lane-aware-tap-delivery.md` decision log
(2026-07-23 entry).

**#448 — row-input belt margin (2026-07-25)**: the INPUT-side
counterpart, and it looks like the attribution
[#435](https://github.com/storkme/spaghettio/issues/435) was missing.
A row of N consumers sharing one input belt provisioned at *exactly*
its aggregate demand starves its TAIL machine permanently: inserters
pick greedily head-first, every machine buffers deeply, and at 100%
belt utilization there is no surplus left to reach the end of the row.
Per-machine sim dumps on `chain-ec15` show the EC row (6 machines ×
7.5/s copper-cable = **45.00/s** against an express belt's **45.0/s**
nominal) holding cable 42 → 34 → 20 → 6 → … → 2, the tail machine in
`item_ingredient_shortage` with 20 iron plates idle beside it, and an
upstream cable producer `full_output` with 32 cable stuck — i.e. the
exact "1 output-blocked, 1 ingredient-starved, 13 working" census the
declared-level sweep reported at EVERY level, and the exact ~5.3%
level-invariant residual recorded above as "currently unattributed and,
since the recalibration, unwarned". It is now warned:
`validate::inserters::check_row_input_belt_margin`
(`row-input-belt-margin`, Warning) groups input inserters by the
`row:<recipe>:belt-in:<item>` segment they pick from (tile-adjacency
clustering, machines attributed through their own inserter — the same
identification `check_row_output_lane_budget` uses), sums
`resolve_row_spec` × `utilization_for` demand across the row, and
compares against the belt's full both-lane stacked nominal.
**Threshold `demand >= capacity - EPSILON`, grounded exactly at 100%
because that is the measured-failing condition** — no "safe margin"
percentage was invented, so the check is a stated LOWER bound on the
true requirement; a margin sweep (the #431 sweep shape, varying
provisioned margin instead of research level) would tighten it.
Scoped to shared belts (≥2 consumers: with one machine there is no head
and no tail) and to `belt-in` segments only (the k-trunk
`row:<recipe>:trunk:<item>` path provisions k parallel belts, which a
single-belt comparison would systematically false-positive).
A whole-suite sweep (instrumented run over every test) puts it at **32
rows across 8 fixture configs**, every one the identical zero-margin
shape and every one judged genuine: chain-ec15's copper-cable row
(45.00 vs 45.0 express, n=6; shared by three cell-composition tests),
`tier2_electronic_circuit_from_ore` (1 row, 24 electric furnaces ×
0.625 = 15.00 vs 15.0 yellow), the EC-from-ore stress fixtures 30s (5),
30s_decomposed (7), 40s (4), 60s_red (5, n=48 × 0.625 = 30.00 vs 30.0
red), `stacking_ec_60s_red_one_belt_headline` (5) and
`partition_strategy_scoreboard` (4, EC rows at 30.00 vs 30.0 red).
Discriminating, not blanket: sibling rows at 91.7%, 90%, 87.5%, 80%,
60%, 40% stay silent in the same layouts. Zero layout-hash change (3
stress goldens re-blessed for warning counts only).
**Root cause is one comparison operator**: `common::belt_entity_for_rate`
picks the cheapest tier with `rate <= throughput`, so any demand landing
exactly on a tier boundary (15/30/45 at S=1) is provisioned with zero
margin by construction — which is why every single finding reads
`demand == capacity` to the penny. The engine-side fix (strict `<`, or a
measured margin) is deliberately NOT in this change: it needs the margin
sweep first, and it would move geometry everywhere.
NOT caught: the `chain-mil5ore` and `mega-chain-pu4raw` tail-starvation
instances — their belts sit at 5.5% and 27% utilization, so whatever
starves those rows is a different binding constraint and is still
unattributed. **(2026-07-26: `chain-mil5ore` is struck from this list —
it was never starving. Re-measured at `--warmup 288000` it runs at
**+0.7%, 146/146 machines working**; the −28.7% was a buffer-fill
transient. `mega-chain-pu4raw` stands, and was independently re-measured
at the same long warmup: −21.0%, byte-identical to its default-warmup
run, so it is genuinely bound by something. See §"Default warmup is too
short for deep chains".)**

**`rfc-047-lane-aware-tap-delivery.md` close-out (2026-07-22)**: made
delivery **lane-aware** so belt stacking raises rate CEILINGS, not just
belt tiers. Leg A: the lane-rate walker's convergence-phase splitter model
was physically false (pooled lanes — real splitters preserve them) —
replaced by `splitter_output_rates_convergence`, exposing
[#334](https://github.com/storkme/spaghettio/issues/334) (two
lane-imbalanced balancer-library shapes, carved out with a fix tripwire);
the mechanics doc's I5 was backwards (inserters drop the FAR lane — code
was always right). Leg B: RFC-046's stacking-blind row-split cap was
fragmenting rows at S>1 and manufacturing sideload overloads — fixed at
the root; a late sideload check now refuses multi-producer single-trunk
over-cap shapes by name (exposed 38 pre-existing silent S=1 overload
errors in a fixture that never asserted on them; (n,1) merge-tap is
unwired, [#336](https://github.com/storkme/spaghettio/issues/336)).
Leg C: the fan-in wall scales ×S on geometry-grounded credits — EC@6/s
legendary yellow refuses at S=1 and builds clean at S=2, and the original
legendary-express@60 headline landed
(`stacking_ec_60s_express_legendary_s2`;
[#335](https://github.com/storkme/spaghettio/issues/335) tracks one
unreached furnace bank). Three falsified premises decision-logged. Zero
golden re-blesses across the arc. Full trail:
[`rfc-047-lane-aware-tap-delivery.md`](rfc-047-lane-aware-tap-delivery.md).

## Open tracking issues (layout quality)

- [#456 flow-preserving compaction / the spaghettifier](https://github.com/storkme/spaghettio/issues/456) — design split into competing [RFC-055 compact linear chains](rfc-055-compact-cell-chain.md) and [RFC-056 folded chains](rfc-056-folded-cell-chain.md); both make validated cell rotations first-class and share one measured decision gate
- [#135 balancer templates are oversized](https://github.com/storkme/spaghettio/issues/135) — main compaction lever
- [#311 output merger over-commits a single final belt; lane-throughput check never visits merger tiles](https://github.com/storkme/spaghettio/issues/311) — gates >45/s headline claims
- [#312 consumer-clamped fan-in refusal bites much earlier at high build quality](https://github.com/storkme/spaghettio/issues/312) — S=1; the wall now scales ×S with stacking (RFC-047 Leg C)
- [#335 one unreached furnace bank in the legendary-express@60 fixture](https://github.com/storkme/spaghettio/issues/335)
- [#336 (n,1) merge-tap unwired; late sideload check refuses those shapes by name](https://github.com/storkme/spaghettio/issues/336)

(Audited 2026-07-21: #65, #68, #136, #310 — previously cited here — are all
closed. 2026-07-24: #334 closed — the (7,3)/(7,4) lane skew is ACCEPTED as a
documented limitation (user call), guarded by `balancer_lane_audit`'s
KNOWN_IMBALANCED tripwire; #266's (5,8) MX1 limit accepted the same way,
guarded in `balancer_classify`. Both revocable on re-bake or field failure.)

## Deferred tooling tasks

Test-suite time recovery (audited 2026-07-19, pick-up notes per item in
[`test-suite-followups.md`](test-suite-followups.md)): committed STRESSGOLD
baseline goldens landed 2026-07-19 (`SPAGHETTIO_STRESS_GOLDEN=check|bless`,
see `crates/core/tests/goldens/stress/README.md` — host-cache-relative,
opt-in, not CI-enforced); CI nextest parallelism re-enable via
timeout-ceiling bumps (~5 min/push, experiment already documented in
`.config/nextest.toml`); `[profile.test]` opt experiment for SAT/A*-heavy
tests (measure before adopting).

## Sim-harness measurement integrity (2026-07-22)

The #357 investigation inverted itself: **every "clean-but-failing" sweep
fixture was a harness artifact, not a layout defect.** Root cause: feed-rig
bank chests from adjacent rigs overlapped on one tile (`create_entity` in
script mode stacks entities silently) and cross-fed ores; iron furnaces
smelted the stray copper and the wrong-item plates permanently plugged
dead-end belt-ins (mechanics **I11**: inserters refuse items the destination
can't accept — one contaminant item plugs a lane forever). With the kit
fixed (PR #362): logistic 1.05/s, military 1.00/s, ec10 10.00/s, automation
1.00/s — **the whole solid sweep PASSES at plan** and #352/#357 closed. Two
further artifact classes fixed en route: buffer-fill transients read as
convergence (`--warmup` steady-state probes) and 20-second snapshot rates
for intermediates (trailing-window). The kit now self-audits
(`kit_errors` ⇒ verdict NO DATA); measurement semantics + forensic playbook:
[`sim-harness-forensics.md`](sim-harness-forensics.md). Baselines re-blessed
clean-kit; a parity re-bless (post-#378 tech-state keying) is in flight.

### Default warmup is too short for deep chains (2026-07-26)

The buffer-fill artifact class above is **not closed** — the `--warmup`
escape hatch exists, but the *default* does not reflect it, and recorded
numbers taken at the default have been wrong in the same direction.

`chain-mil5ore-d2` is recorded across this repo as a **FAIL at −28.7%**
(RFC-054's calibration corpus, RFC-051 close-out). Re-run unchanged at
`--warmup 288000` (80 game-minutes) it measures **+0.7%, 146/146 machines
working, PASS**. Nothing about the layout changed; the measurement started
before the factory finished filling. The native meter shows the same shape
on `chain-mil5plates-d0`: −38.4% at a 2-minute warmup, +0.7% converged.

Consequences, and they are open:

- **Any recorded deficit taken at the default warmup on a multi-stage chain
  is unproven.** Sweep warmup and watch the number: a real deficit is flat,
  a transient is not
  (`cargo run -p spaghettio_meter --example warmup_sweep -- <label>`).
- **Suspect, not yet re-measured**:
  [#453](https://github.com/storkme/spaghettio/issues/453) (USP@2, −57.0%,
  described there as the single highest-value unknown left in the
  composition path) and
  [#437](https://github.com/storkme/spaghettio/issues/437) (PU@4, −27.3%).
- **RFC-054's KC1 cannot be re-evaluated until the corpus is re-measured**,
  because its rank half grades against band assignments that are now known
  to contain at least one error. Re-banding must be justified by the oracle
  alone, never by agreement with the meter.
- **Convergence detection is a floor, not a ceiling.** Both instruments
  report converged for `chain-mil5ore-d2` at a 40-minute warmup, where it
  still reads −13.8% against a settled −1.3%. Stability windows cannot
  distinguish steady state from a large factory filling slowly and
  smoothly; a generous fixed warmup is currently the only defence.

### Fluid byproducts stall multi-output machines (2026-07-26) — FIXED IN CODE, RE-MEASUREMENT PENDING

Mechanics **F13**: a machine blocked on **any** output fluid box stops
crafting entirely, including the products that *are* wanted. Multi-output
recipes therefore need every output to reach a consumer, a surplus exit, or
a void.

Observed in `mega-chain-usp2raw`, distinct from #471's serial splitter
allocation:

- 21 `basic-oil-processing` refineries saturate the petroleum-gas network
  (100/100 across 276 pipes).
- The 3 `advanced-oil-processing` refineries must also push petroleum-gas
  into that full network, so they block — **11 of 24 refineries read
  `full_output`**.
- Blocked, they produce no **heavy-oil**, of which they are the only source.
  Heavy-oil and light-oil appear on **zero pipes** factory-wide.
- The lubricant plants sit in `fluid_ingredient_shortage` holding 5 units;
  lubricant runs at ~10/100 and #453 records it at **−18.5%**.
- `surplus_exits` is empty. Petroleum-gas is terminal and cannot be cracked
  further, so the options are consume it, void it, or size advanced-only
  against petroleum demand rather than mixing both paths.

The initial diagnosis was incomplete: the LP *did* credit advanced oil's
petroleum before sizing basic oil. The unsafe shape appeared at the
fractional-plan → physical-layout boundary: a mixed 18.815-basic /
2-advanced plan became 21 + 3 whole refineries across three replicas, whose
combined petroleum capacity could saturate the network and block advanced
processing.

Fix [#476](https://github.com/storkme/spaghettio/issues/476): free recipe
selection now treats the oil paths as exclusive. If advanced processing is
required, the solver re-solves without basic processing; USP@2 becomes
advanced-only and its unavoidable heavy-oil excess is explicit. The mega
adapter now preserves that surplus through its sub-solve, boundary
translation, and chain placement, producing an entity-verified heavy-oil
perimeter pipe. The solver regression and the real ignored USP composition
gate pass. Keep this entry open until a long Factorio re-measurement confirms
the refinery/lubricant stall has disappeared and quantifies the remaining
#471 deficit.

### F10 segment-extent limit: not currently breached, and unchecked

Mechanics **F10**: a fluid segment whose tile extent exceeds 320×320 does
not flow at all. Composed mega-chains span ~2,200 tiles and the engine emits
**zero pumps**, so this looked like a candidate failure mode. It is not, at
present: real segments in `mega-chain-usp2raw` top out at **49×109 tiles**
(21 segments, none over the limit), because PTG runs break the trunks up.

**The validator has no segment-extent check**, so this is a latent risk
rather than a live defect — a future layout with longer uninterrupted fluid
runs would fail silently and totally. `scripts/fluid_segment_extents.py`
computes segments from a sim-state dump if it needs re-checking.
