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
([#392](https://github.com/storkme/spaghettio/issues/392) — accepted
never runs full validation). Measured-at-plan claims live in
the sim-verified registry (geometry-hashed, checked-in): currently
AC-from-plates (1.00/s, 8/8 working). EC-row geometries measure −8%
under tech-state parity — the validator's inserter-item-throughput
warnings turned out RIGHT under declared capacity (the Phase-1 "15.0/s
exact" was a pre-#378 researched-bonus artifact;
[#383](https://github.com/storkme/spaghettio/issues/383) has the
forensics) — re-measure after RFC-049 Phase 3 inserter sizing
([#381](https://github.com/storkme/spaghettio/issues/381)). Full
trail:
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

**`rfc-049-inserter-capacity-research.md` close-out (2026-07-22)**: user-facing **inserter capacity research** param (level 0–7, `inserter_capacity`/`ir=` URL-encoded, sidebar "Inserter research"). Schedule pinned from raw wikitext with 2-fetch reproducibility (bulk 2→12; stack = bulk+4 → 6→16; non-bulk 1→3 via the chain +1 from Transport-belt-capacity-2 → 4) — summarized wiki fetches are BANNED as constant sources (two contradicted each other; the failure mode reproduced live in review). Output belt-drop sides originally scaled linearly (swings × researched hand, with BS3 rounding — healing is exactly `hand ≡ 0 mod S`, non-monotonic: I8b) — **superseded 2026-07-23 (#385)**, see below. Input (belt-pickup) sides stayed at the L0 floor pending measured data — **closed 2026-07-22 (Phase 2, PR #378, #343)**: a 25-cell sim calibration matrix (tech-state-parity harness, all 8 levels for stack/bulk) measured belt→machine intake; `common::machine_feed_rate` now credits hand-ratio rates for non-bulk/bulk (measured conservative, 1.04–2.27× margins) and a measured floor table for stack (its real curve is non-monotone in hand size — dips at hands 7/14 — caught by the #376 adversarial review and confirmed by measurement). L0 bit-identical; the sizing ladder deliberately stays L0 (user-facing density-vs-research trade, undecided). In-game anchor open (user-run; a legendary S=4/L7 export validates RFC-046/047/049 in one import). Full trail: `docs/rfc-049-inserter-capacity-research.md` decision log.

**#385 belt-drop min-form (2026-07-23)**: the RFC-049 belt-drop swing term (`swings × researched hand`) was never checked against the belt's own physical throughput — sim-measured onto yellow (true-S1 world) found stack credited 2–5× over the real 6.50/6.50/7.10/s (L0/L2/L7) and fast over-credited 44% at L7 (9.24 vs measured 6.40). `common::belt_drop_rate(name, quality, stacking, level, target_belt)` gained a `target_belt` param and now returns `min(swing_term, 0.85 × lane_capacity_stacked(target_belt, stacking))` — a stack inserter's flat 12.0/s credit onto a plain yellow belt (S=1, L=0) now credits 6.375/s, and non-bulk's L7 multiplier is sim-corrected 4.0→2.67. This deliberately breaks RFC-049's own L0-identity baseline for belt-drop (the 12.0 credit was never physically real) and RFC-046/049's "no recalibration" pattern for the output side specifically — the same "measured, never derived" discipline kill 2 already required for the input side. Threaded through the ladder (`size_belt_drop_side`/`size_side_output`, which lost their now-incorrect `stacking≤1 && level==0` shortcut) and the validator (`belt_drop_throughput`, which derives the drop tile's belt tier from the layout, falling back to yellow when none is found). One e2e fixture's expected inserter count changed (2→3 stack inserters, `fluid_multi_input_sulfur_output_uses_extra_column`); two constants-identity assertions updated (L7 non-bulk ×4.0→×2.67). Full suite clean (lib 798 / e2e 60 / netflow parity 10), clippy `--lib` clean, WASM rebuild clean, zero golden re-blesses. Full trail: `docs/rfc-049-inserter-capacity-research.md` decision log (2026-07-23 entry) and `docs/sim-harness-forensics.md`.

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

- [#135 balancer templates are oversized](https://github.com/storkme/spaghettio/issues/135) — main compaction lever
- [#311 output merger over-commits a single final belt; lane-throughput check never visits merger tiles](https://github.com/storkme/spaghettio/issues/311) — gates >45/s headline claims
- [#312 consumer-clamped fan-in refusal bites much earlier at high build quality](https://github.com/storkme/spaghettio/issues/312) — S=1; the wall now scales ×S with stacking (RFC-047 Leg C)
- [#334 two lane-imbalanced balancer-library shapes](https://github.com/storkme/spaghettio/issues/334) — carved out of the RFC-047 convergence walker with a fix tripwire
- [#335 one unreached furnace bank in the legendary-express@60 fixture](https://github.com/storkme/spaghettio/issues/335)
- [#336 (n,1) merge-tap unwired; late sideload check refuses those shapes by name](https://github.com/storkme/spaghettio/issues/336)

(Audited 2026-07-21: #65, #68, #136, #310 — previously cited here — are all
closed.)

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
