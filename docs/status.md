# Project status ledger

**Status (2026-07-21)**: moved out of `CLAUDE.md` so the agent-context file
sticks to process; this is the canonical home for capability status now.
Update this file (not `CLAUDE.md`) when a tier's status changes or an RFC
closes out. Per-topic backlogs stay in their own `*-followups.md` docs; this
ledger is the cross-cutting view.

Fully re-audited 2026-07-21 (fresh `science_gauntlet` run + default-suite
sweep + issue-state check); per-row history trimmed to current status —
the evidence trails live in the owning RFC decision logs.

**Validator trust table** (2026-08-07): per-check consequence, trust basis,
calibration receipts, and graduation preconditions now live in
[`validator-trust.md`](validator-trust.md) — the registry PRs must update
when changing a check's severity, category set, or selection participation.

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
| 2 | `electronic-circuit` | 2 recipes, 2 solid inputs | SOLVED, 0 errors; **#519 recalibration (2026-07-31)** surfaces the tail-starvation the blessed ec10 sim baseline has recorded since 2026-07-22 (FAIL −50%, #352): from-ore pins 3 input-rate-delivery + 1 belt-margin, from-plates pins 1. Stress-gated at 20/22/23/30/35/40/s (yellow) and 60/s (red) from ore |
| 3 | `plastic-bar` | 1 recipe, 1 fluid + 1 solid input | SOLVED — clean, incl. from crude; sulfuric-acid, heavy-oil cracking, and multi-machine advanced-oil-processing also gated at this tier |
| 4 | `advanced-circuit` | 5+ recipes, mixed solid/fluid | SOLVED — from plates green with 1 known belt-detour warning (the AC last-segment loop, measured whole by RFC-065 slice 2's phantom-cut fix 2026-08-06; root-cause open); from ore (AM2) **validator-clean since RFC-060** (the horizontal-stack candidate wins strictly-better and deletes the long-standing input-rate-delivery residual), and the [#519](https://github.com/storkme/spaghettio/issues/519) recalibration (2026-07-31) now REPORTS that flux honestly: the from-ore AM2 fixture pins 7 input-rate-delivery warnings (11 until 2026-08-15 — #644's phantom-UG-source walker fix cleared 4 fabricated reads) — the check agreeing with `ac@5`'s sim measurement (75% of plan at what was then E0/W0) instead of hiding it. Partitioned 4/s + 5/s (+2 pinned) and horizontal-stack 7/s stress-gated. |
| 5 | `processing-unit` | Deep chain, multiple fluids | SOLVED, 0 errors — from ore (AM3, 2/s) pins 13 input-rate-delivery (32 until 2026-08-15 — #644's phantom-UG-source walker fix cleared 19 fabricated reads; the meter's 85.6%-of-plan reading stays open on #644 as the zero-headroom class); horizontal-stack gated at 2/s (pipe bypass) and 25/s (pole coverage). Higher-rate partitioned strategies still have junction + starvation issues — `partition_strategy_scoreboard_extended`. |
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
inserter-item-throughput warnings — which, as of 2026-08-21 (offpath
item 9, owner call), are **validator-unreported**: the never-sim-anchored
check pair was deleted, so this residual is tracked here and by the
meter, not by `validate()`.

### Beyond the ladder — capabilities the default suite also gates

The tier table understates current capability; these are all regression-gated
on every push:

- **Self-loop / byproduct chains** (net-flow solver): Kovarex enrichment,
  uranium processing (surplus export + voider variants, voider purity),
  pentapod-egg, fish-breeding, and bacteria self-loops. (Kovarex carries
  1 known belt-detour warning since RFC-065 slice 2 measured its catalyst
  return line whole — 2.5×/33-excess, inherent loop topology; whether
  catalyst returns deserve their own calibration class is noted follow-up
  in the RFC-065 log.)
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
  (RFC-047): rate ceilings scale ×S; at S=2 the EC@60/s red-from-ore
  config's EC family fits one stacked belt (in-fixture TIER-SELECTION probe
  — **not** a per-tile capacity audit, corrected 2026-08-07, see
  [`rate-stamp-semantics.md`](rate-stamp-semantics.md); per-tile standing
  rests on `check_lane_throughput` plus the 96.0%-of-plan sim; the
  belt_flow/belt_structural disagreement was arbitrated 2026-08-15,
  #632 B5 — belt_flow dispatched, twin deleted),
  and the legendary-express@60 headline is gated
  (`stacking_ec_60s_express_legendary_s2`).
- **Rate headroom caveat (S=1 only)** — **substantially retracted
  2026-08-07** ([`rate-stamp-semantics.md`](rate-stamp-semantics.md), PR
  #601). This read "output above one belt's capacity is over-committed onto
  a single merger belt, and the lane-throughput check doesn't visit merger
  tiles" ([#311](https://github.com/storkme/spaghettio/issues/311), parked →
  cluster [#527](https://github.com/storkme/spaghettio/issues/527)). Both
  halves fail: the "over-committed" reading came from comparing
  `PlacedEntity::rate` — an *aggregate*, never per-tile flow — against one
  belt's capacity, and the tiles so flagged carry 7.5–9.0/s where stamped
  60/s; and both lane models do return rates for every merger tile. Any
  residual #311 defect must be re-argued from a walked model or a sim.

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
(parked → cluster [#527](https://github.com/storkme/spaghettio/issues/527))
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

The full close-out narratives (855 lines, 2026-07-13 .. 2026-08-05,
with embedded corrections through 2026-08-08) are
archived VERBATIM in
[`archive/status-rfc-closeouts-2026-jul-aug.md`](archive/status-rfc-closeouts-2026-jul-aug.md)
(#632 docs sweep, 2026-08-15) — each entry's canonical trail is its
RFC's own decision log. Index, verdict phrases quoted from the entries:

| Close-out | Date | Verdict (as recorded) |
|---|---|---|
| RFC-071 evidence-calibrated selection | 2026-08-23 | COMPLETE 2026-08-24 (#711–#717 all merged) — the #710 calibration matrix brought to 20/35 Factorio-vetted (recipe-productivity pin closes RFC-064 item 7's fix, twice sim-anchored, #714; 7-row re-measurement campaign clean); first evidence table lands (#713: route-severing validator categories appear ONLY on sim-broken rows — zero on all 20 working factories); gear@20 75% root-caused to the cell chain's yellow-hardcoded final drain (NOT the opening bridge story — falsified) and fixed at plan, sim 20.0/20.0 PASS (#715); `RouteSevered` error class ships (#716: ec35/ec40 flip from measured-dead 0/s winners to 8.0/6.75 per s, sim-anchored; the policy kind-table found decorative and wired live; ec35 superseded 2026-08-24 by RFC-069 Phase A1's rescue — 93.7% delivered, see the residual-warnings section); verified-geometry-first ordering on best-error-free (#717, gear@20's geometry graduated into cell-sim-registry). Open: ird/margin weights (evidence straddles), phantom-feeder report-only check, >45/s single-drain ceiling. Follow-up 2026-08-24: bank regenerated on merged main — first full 35/35 coverage (gear@20 measures 100% at plan in-bank), bank fingerprint committed (`crates/core/data/calibration-bank/`) with a CI probe gate so geometry drift on calibrated rows fails at PR time |
| RFC-070 one selection loop (Phases 0–2c) | 2026-08-22 | COMPLETE — v2 `SelectionPolicy` ships production selection (#707), generation-1 loop deleted (#708: 2,101 deleted / 1,223 added, net −878); K70-1 PASSED live at the shadow gate (140/140 winner+stage); zero divergences; 16 PRs; en route: two e2e harness fossils fixed (#699), ec30 0/s jam root-caused and, via the (3,2) balancer restore, sim-anchored at 97% of plan (#701/#706, 29.09/30 delivered — not 'cleared': the RFC anchor entry carries the caveats), gear@20 75%-of-plan diagnosed (#700; root-caused and fixed under RFC-071 — see its row); Phase-3 calibration executed as RFC-071, RFC-068 P1 open |
| RFC-065 connectivity-IR Phase 2 | 2026-08-05 | killed by measurement — premise falsified on both paths; RFC stays Active (Phase 1 slices) |
| RFC-062 multi-target outputs | 2026-08-01 | PARTIAL — engine correctness lands; the final gate's two measurement kill criteria both FAIL as measured |
| RFC-063 compaction primitives | 2026-08-01 | CONCLUDED — all three phases adjudicated (Phase A killed at A0; Phase C cleared its escalated bar without funding production wiring) |
| RFC-058 band packing | 2026-07-31 | CONCLUDED — kill criterion 1 fired in phase 4; flag-gated builder stays as the falsification record |
| RFC-059 DI claim order | 2026-07-31 | CLOSED — default is `Downstream`, flipped on in-game measurement (supersedes the earlier P0-tie-break verdict recorded above it) |
| #520 half-rate factory incident (+ #526 geometry repair) | 2026-07-31 | validator shipped a half-rate factory; reporting incident ten; #526's first fix was itself wrong and the sim caught it |
| RFC-057 dense repacking (snake fold, multi-fold) | 2026-07-29/30 | both Factorio-verified at plan; RFC ACTIVE at the time |
| RFC-053 direct-insertion cells (Phases 0-4, fluid producers, 5th corpus pair) | 2026-07-25/26 | landed incrementally; DI ON BY DEFAULT 2026-07-26; all 5 sims PASS |
| RFC-052 oil mega-cell | 2026-07-24 | close-out, Phases A/B/C — fluid subgraphs compose as uncropped mega-cells |
| #385 belt-drop min-form + row-output lane budget | 2026-07-23 | swing-term check grounded; second half closed |
| #448 row-input belt margin | 2026-07-25 | input-side margin check landed (one comparison-operator root cause) |
| RFC-047 lane-aware tap delivery | 2026-07-22 | close-out |
| RFC-048 cell composition Phase 1 | 2026-07-22 | close-out (PR #365) |
| RFC-049 inserter capacity research | 2026-07-22 | close-out — user-facing param, levels 0-7 |
| RFC-051 cell-composition integration | 2026-07-22 | production path ON BY DEFAULT |
| RFC-044 machine modules | 2026-07-21 | close-out, all 4 phases |
| RFC-046 belt stacking | 2026-07-21 | close-out — user-facing S in {1..4} |
| RFC-043 pole-band thinning | 2026-07-20 | close-out — quality-aware |
| rfc-build-quality | 2026-07-20 | close-out — user-facing build quality |
| rfc-inserter-sizing | 2026-07-13 | close-out — ladder-sized bus inserters |

## Open tracking issues (layout quality)

- **`ec30-am2-ore` at production defaults is sim-anchored at 97% of plan, not claimed fixed (#701, 2026-08-22).** `electronic-circuit@30` on `assembling-machine-2` from ore, with the belt tier left to the engine and `LayoutOptions::default()`, delivers **29.09/s vs 30.00/s planned (−3.0%)** and produces **28.91/s (−3.6%)** in the Factorio headless anchor; it converges with **+0.6% drift**. The export revived **2,071/2,071 ghosts**; the machine census is **161 working / 6 full-output / 3 ingredient-shortage**. The layout carries **input-rate-delivery×7, belt-detour×1, and row-input-belt-margin×1** warnings (OVERALL WARN), so this is a measurement of a warned layout, not a clearance claim. The residual is **3%, not 16.7%**: it sits inside the validator-warned `input-rate-delivery` class, not the balancer's partial-input behaviour. The meter's 25/30 was pessimistic for this question; the sim anchor removes the old 0/s failure mode but does not establish that the path is fixed. This production-default fixture remains distinct from the stress-EC fixtures below, which force `Some("transport-belt")` through `run_e2e`.
  - **Sim anchor receipt:** [`ec30-am2-ore-32-sim-receipt.json`](../artifacts/ec30-am2-ore-32-sim-receipt.json), Factorio 2.0.77, `--warmup 432000 --speed 32`; electronic-circuit delivered **29.09/s**, produced **28.91/s**, converged, and the full report hash is recorded there.
  - **Selection/parity receipt:** the re-blessed `parity_corpus_baseline.json` moves only the two electronic-circuit-from-ore rows: horizontal-stack wins via `scoped-pairwise` where enabled, while `hs-off` selects clean native via `best-error-free`; the am1/di-off cells retain native with a stage-only `best-accepted` → `best-error-free` change.
- [#456 flow-preserving compaction / the spaghettifier](https://github.com/storkme/spaghettio/issues/456) — design split into competing [RFC-055 compact linear chains](rfc-055-compact-cell-chain.md) and [RFC-056 folded chains](rfc-056-folded-cell-chain.md); both make validated cell rotations first-class and share one measured decision gate
- [#135 balancer templates are oversized](https://github.com/storkme/spaghettio/issues/135) — **RFC-063 Phase A killed at A0, 2026-07-31**: the ≥25% bbox bar this issue was funded to reach is unreachable against verified community-best balancer references (≈8.1% gate-fixture / ≈5.9% holdout measured ceiling) — no longer "the main compaction lever". Low-risk maintenance residue (regenerate the likely-stale `(4,5)` template; correct stale doc-comment baselines) spun off to #551, not funded as arc work.
- [#507 RFC-058 band-packing tracking](https://github.com/storkme/spaghettio/issues/507) — RFC-058 concluded 2026-07-31 (KC1 fired in phase 4, −27.0% vs a −33.0% bar); the flag-gated `bus::bands` builder stayed in-tree as the falsification record until **2026-08-20, when the owner extended the #632 A2 precedent and it was DELETED** (offpath-code-followups Tier 2; the record is RFC-058's decision log). RFC-063 Phase C (concluded 2026-08-01) re-ran the same builder on DI-composed input and cleared an escalated −40.0% bar on 2/3 fixtures — see `rfc-063-compaction-primitives.md`'s decision log — without funding production wiring. RFC-064 Phase 3 — the last arranged consumer of this scaffolding — concluded-failed 2026-08-03, which is what cleared the way for the deletion.
- [#527 parked: bus high-rate scaling cluster](https://github.com/storkme/spaghettio/issues/527) — #311 #312 #335 #336 #345, closed not-planned 2026-07-31; all real, none fixed; revisit at the cell-interface RFC. #311's merger wall was evidenced by the stamp-vs-capacity comparison, **falsified 2026-08-07** (PR #601, `rate-stamp-semantics.md`); it no longer gates >45/s headline claims on that evidence.
- [#526 DI cell geometry: belt-to-belt lift picks upstream of its only feed](https://github.com/storkme/spaghettio/issues/526) — **geometry repaired 2026-07-31** (narrative: the #520/#526 entry in [the close-out archive](archive/status-rfc-closeouts-2026-jul-aug.md)): `stamp_di_bridge` now refuses a bridge that can't clear a DI cell's LAST producer drop, rather than shipping a partial-throughput layout. Corpus-swept: changes zero shipped layouts (native already won everywhere via #524's gate). Still open: the policy question #520 raised, whether `di_choice` should require more than validator parity before displacing native
- [#519 flux blind spot](https://github.com/storkme/spaghettio/issues/519) — **walker recalibrated 2026-07-31**: consumption decrement along rows (plus four model-artifact fixes found by fixture bisection) makes `input-rate-delivery` report tail starvation the sims measured (`ac@5` now E0/W7 at the exact machines its sim census showed empty; was E0/W0 at 75% of plan). Still open for: merge-aware demand attribution (the map over-attributes up every merge branch, so demand-weighted external seeding is consistency-gated) and folding flux into candidate SELECTION — **done 2026-08-07, the exemption is lifted**; `input-rate-delivery` now counts in `selection_warning_count`. Receipts (PU 68.2% -> 102.0% delivered, big-electric-pole holding at plan, drift adjudicated 6 fixtures -> 2) live in [`validator-trust.md`](validator-trust.md) hole 2 and nowhere else. **The sim-anchor precondition is substantially met (2026-08-07)**: the layout the un-excluded ranking picks measured **102.0% of plan** (converged, 4 checkpoints, kit clean — the sound, positive direction), and the flagged layout's shortfall is confirmed qualitatively in-client (owner observed the starved EC belt, 4 producers vs 8 consumers) though its 68.2% rate figure stays provisional (min-checkpoint run, unreconciled with #591's 90–98% note) — receipts and the upgrade path in [`validator-trust.md`](validator-trust.md). Lifting the exclusion was blocked on the #311-class over-capacity gap, which is **void as of 2026-08-07** (PR #601 — the comparison was a category error). The lift then hit, and cleared, a second blocker of its own. **Its status is tracked ONLY in [`validator-trust.md`](validator-trust.md) hole 2** — deliberately not restated here, because this entry previously carried three copies of it that drifted apart. Gauntlet/scoreboard warning totals recorded before 2026-07-31 pre-date the recalibration and undercount.
- **stress-EC high-rate half-plan (2026-08-01, RFC-064 Phase 2 sim)**: `stress_electronic_circuit_30s_from_ore` and `stress_electronic_circuit_60s_red_from_ore` measure **~50% of plan** (15.15/15.15 vs 30; 30.5/30.0 vs 60) and `tier2_electronic_circuit` ~42% below (5.77/5.81 vs 10) — **superseded 2026-08-07**: the `input-rate-delivery` lift re-ranks that config onto a different winner, and the re-measured layout produces **9.09/s vs 10 planned (91% of plan)**, i.e. **58% -> 91%**. Still a FAIL, and the residual is a *uniform* ~10% across BOTH stages (copper-cable 90.0%, EC 90.9%) — by the y=mx+c reading in [`sim-harness-forensics.md`](sim-harness-forensics.md) that is a SHARED constraint, not one stage bottlenecking; 1 machine sits in `item_ingredient_shortage`, 16 working, kit clean and converged over 8 checkpoints. Root cause not yet chased. The two stress-EC figures were unaffected by *compaction* on **both native and compact**, and were read at the time as a pre-existing solver throughput ceiling for high-rate EC-from-ore (the #519 `input-rate-delivery` warnings predicted it). **That reading is SUPERSEDED 2026-08-07/08**: re-measured post-lift they land at **92.1%** and **90.7%** delivered, so the ~50% was overwhelmingly the same selection bug, not a solver ceiling. What survives is a real ~8-10% residual on both, matching the zero-headroom shape below. Not a compaction regression; open defect, see `rfc064-phase2-followups.md` §1.
- **Bio/fluid self-loop fixtures unmeasurable in the harness rig (2026-08-01)**: `bacteria_self_loop_regression` measures 0/s (`no_fuel`), same class as pentapod/fish and the `fluid_ingredient_shortage` fixtures (sulfuric/heavy-oil). Validator-clean but sim-dead — a rig + validator-coverage gap; on the sim skip-list until fixed (`rfc064-phase2-followups.md` §2).
- **RFC-064 Phase 2 Stage B verdict (2026-08-01)**: never-worse HOLDS on the measurable subset → evidence supports `compact_layout` default-on (representative-subset scope; see `rfc064-phase2-followups.md`). Sim tuning adopted same day: speed 32 + deep warmup 108000 (30 game-min), a 2.7× cut from 80. **MOOT (2026-08-14, #632 A2, owner call)**: `compact_layout` and `bus::compaction` are deleted — there is no flag left to default on. The 68-sim Stage B campaign this verdict rests on was itself parked, never re-run to completion; the never-worse finding stands as a historical measurement, not an open recommendation.
- **Deep-chain throughput deficits, sim-measured 2026-08-06 (RFC-064 productivity stack tip)**: with the research-productivity axis correctly declared, `processing-unit@1/s` measures **68.2% of plan** and `advanced-circuit@5/s` **83.3%**, while `electronic-circuit@10/s` is exactly 100% — so the deficit is depth-specific, not general. The signature is a **uniform** ratio across every stage of the chain (PU run: copper-cable .6874, copper-plate .6878, EC .6875, iron-plate .6874, plastic .6875, target .6818) plus ~1/3 of machines in `full_output`, i.e. a throughput/distribution ceiling, NOT a productivity-modelling error — declaring the axis is what made these numbers interpretable, it did not cause them. AC is bit-identical declared-or-not despite a measurably smaller plan — **no longer true as of 2026-08-21** (#689 W1c): `advanced-circuit@5/am2` from ore exports 2134 entities undeclared and **2125 declared** at `plastic-bar=0.10`, same 137×78 dims. The engine has moved since 2026-08-06; re-measure before reusing this equivalence anywhere. **PU root-caused 2026-08-07** — the eyes-first gate paid out exactly as intended: the owner served the fixture, saw the green-circuit belt into the PU rows starved (4 EC producers feeding 8 consumers; the sulfuric-acid backup was a downstream symptom), and the validator turned out to have flagged precisely this — three `input-rate-delivery` warnings naming the starving machines, which `selection_warning_count` was excluding from candidate ranking. With the exclusion lifted, the re-ranked PU layout sims at **102.0% of plan** (converged, 140/145 census working) — the lift was parked because it also re-ranks `stacking_ec_60s` onto a supposedly "physically impossible" winner — **retracted 2026-08-07** (PR #601): that verdict compared an aggregate stamp to one belt's capacity and has zero true positives, so THAT objection is void — but the lift has its own separate status, tracked ONLY in `validator-trust.md` hole 2. **AC@5/s ALSO FIXED by the same lift, measured 2026-08-07: 83.3% -> 99.7% of plan** (4.98/s vs 5.00 planned, PASS, converged, kit clean, 109 machines working) — and every stage lands at plan, not just the target (cable +0.0%, EC +0.0%, plastic -0.3%, petroleum +0.6%). It was never eyeballed; it did not need to be, because the deficit had the same root as PU's. So the deep-chain deficit class is substantially the `input-rate-delivery` exclusion: PU 68.2% -> 102.0%, AC 83.3% -> 99.7%, tier2_electronic_circuit 58% -> 91%. **The stress-EC ~50% pair HAS now been re-measured post-lift (2026-08-07/08)** and it moved the same way: `stress_electronic_circuit_30s_from_ore` ~50% -> **92.1% delivered** and `stress_electronic_circuit_60s_red_from_ore` ~50-51% -> **90.7% delivered** (both kit-clean, converged, warmup 432k; banked at `~/spaghettio-corpora/postlift-2026-08-07/`). So the historical ~50% was overwhelmingly the selection bug, not a throughput ceiling — but a real ~8-10% residual remains on both, and both fixtures have all four stages exactly zero-headroom, which is the shape the bullet below predicts. **2026-08-15 (#644, PR #648)**: the swap-era 140/218 lane-throughput Error readings on this pair were retracted as walker phantom-UG-source artifacts (corpus now 0 lane errors); the phantoms had also steered candidate selection for the three days they existed, and the fix flips both winners BACK to these banked layouts (entity-count-exact 3369/4967), so the 92.1%/90.7% receipts attach to what the fixed engine ships. The residual's fix campaign is RFC-069 (achievable-duty provisioning); its Phase-0 additions: ec22 delivers 99.4% (the 22-vs-30 boundary is real under measurement) and tier5-PU@2 axis-declared delivers 95.6% (the meter's 85.6% was an axis-mismatched export). **RFC-069 resumed 2026-08-24 as the trunk/tap-provisioning campaign — Phases A1+A2 landed (#720, #721)**: ec35/ec40/tier5's blocker re-diagnosed as the coprime balancer-trap class (tier5's "router block" retracted — same class), the `k1-shape-fix` rescue made reachable on the Pooled path, ec35 flips from the 313-error merge-tap at 22.9% to a 0-error k1 layout **sim-measured 93.7% delivered** (kit-flagged; meter 95.7%); A2 adds multi-consumer enrollment (tier5's k1: Refused → Produced/12E) and the held-incumbent migration, which flips **ac45 from sim-dead 0.0/s to an error-free cell-composed layout at 63.7–66.7% non-converged** (a real ~2/3 ceiling — the meter's 97.8% is wrong on the shape; residual joins the cell-drain ledger). **Phase A3 (2026-08-25): the trap class dissolved at the root** — the lane split now consults the stamper's own resolvability oracle and pads unresolvable trunk counts (the "router class" attribution retired: the real defect was zero-height balancer bands from `family_stamp_plan`-Unresolvable shapes). The NATIVE reclaims the class: ec40 flips off the 631-error merge-tap onto a 1-error native, **sim 36.8/40 = 92.0% converged kit-clean** (from 18.5%); ec35's native builds the rescue artifact itself; tier5@0.6 sheds its three trap families. **Campaign COMPLETE 2026-08-25**: Phase B adjudicated GATED by measurement (tier5@0.6 = 13 structural errors for +4.4% — measured-negative; two rescues falsified by experiment; duty stays opt-in with the ec30 99.4%/ec60-red 100.0% receipts standing; forward path = duty-as-candidate + density re-weigh, the RFC's remaining substance) and Phase C shipped (the typed unreachable-rate refusal). ec40's residual 8% is UNDECOMPOSED between zero-headroom (the ec30-family precedent) and the pad's orphan-stub dilution — the duty knob measures byte-identical on the padded natives, so the decomposition needs a stub-less arm (recorded follow-up).
  - **⚠ SUPERSEDED FOR THIS FIXTURE (2026-08-08, #607/#608).** The zero-headroom reading below is **falsified for `tier2_electronic_circuit @10/s`** and left in place only because its *scoping numbers* (47% of stages, the cost model, the pooled-vs-partitioned confound) are still good and load-bearing for other fixtures. What is wrong is the attribution of THIS fixture's ~9-10%: the fast meter shows **nine of the ten copper-cable machines saturated at 270 crafts** over the window with one at 152, i.e. the stage could make plan and the binding constraint was elsewhere. It was the `di-bridge` belt→belt transfer bank, whose 14 long-handed inserters feed the consumer's input belt by hand and therefore load **one lane only** — a ~21.4/s ceiling against 30/s of demand. With #608 crediting that belt honestly, selection ships the bus-lane variant instead, which measures **100.0% of plan headless** (PASS, converged, drift 0.0%) against the bridge's 90.9%. Zero-headroom may still contribute elsewhere; it is not the mechanism here. Do not cite the paragraph below for this fixture.
  - **tier2's residual 9% is ROOT-CAUSED (2026-08-07) and is a different class: ZERO-HEADROOM INTEGRAL MACHINE COUNTS.** copper-cable plans at *exactly* 10.0 machines (10 x 3.0/s = 30.0/s against a 30.0/s requirement), so every machine must sustain 100% duty forever to hit plan — inserter swing, belt gaps and momentary pauses make that unreachable, and it measured 90% duty (27.0/s). EC is stoichiometrically downstream (3 cable per EC), so 27/3 = 9.0/s against 9.09/s measured — the arithmetic closes. The *uniform* look is one zero-headroom stage propagating, not two independent shortfalls. Contrast electronic-circuit itself: count 6.667 rounds up to 7 placed, giving 10.5/s against 10.0/s needed — 5% headroom that absorbs the same duty loss. **No validator fires, correctly**: nothing is structurally wrong, the plan is satisfiable in theory and not in practice. The fix direction is solver/placer sizing for *achievable* duty rather than nominal 100% when a count lands exactly integral; not attempted here. **Scoped 2026-08-07** (40 fixtures / 146 stages, solver-derived): **69/146 stages (47%) are exactly zero-headroom, across 28/40 fixtures**, and the near-zero class is just as big — **92/146 (63%) sit under 2%**, e.g. `measure_utility_10s_am3` copper-plate 797.33->798 = 0.08%, with 18 of that fixture's 22 stages under 2%. Cost: a flat *+1 machine when headroom < X%* rule is ~3x cheaper than a multiplicative one (+107 machines at <5% vs +357 at x1.05), because the flat rule adds one machine only where needed while the multiplicative one scales with every existing fleet. Measured entity cost is NOT linear: ~10 entities per bumped machine on a shallow single-row fixture, 50-75 on deep chains (the row layout re-solves and footprint moves non-monotonically). **Crucially, zero-headroom is NECESSARY BUT NOT SUFFICIENT**: every fixture measuring materially below plan has a zero-headroom stage, yet several fixtures whose *target* stage has zero headroom measure 97-107%. The sharpest evidence is `stress_advanced_circuit_partitioned_5s_from_plates`, whose pooled and partitioned variants have *identical* solver output and headroom profile but measure **80% vs 98-100%** — same plan, opposite outcome, because layout strategy is a confound. So zero-headroom is not itself the defect; it is the removal of the margin that would otherwise absorb ordinary routing/inserter loss, which means *reducing the loss* is as valid a fix direction as *adding headroom*. Note PU and AC carried zero-headroom stages too yet were root-caused to the #601 selection bug, so the two defects are historically confounded. Highest-value next check: re-measure `stress_electronic_circuit_30s/60s` post-lift — all four of their stages are exactly zero-headroom, making them the cleanest candidates.
  - **The METER already catches this class, and nothing consumes its verdict (2026-08-07).** Run on the same tier2 layout it reports copper-cable **28.8/s (96% of plan)** and electronic-circuit **9.6/s (96%)** — against the real sim's 27.0/90% and 9.09/91%. Directionally right on both stages, ~5pp optimistic on magnitude, and **in 19 seconds** versus ~10 minutes for the headless run. It sees it because it models inserter swing and lost swings (`inserter.rs`: *"every tick spent here is a lost swing — the starvation signal"*), which is exactly the duty loss a zero-headroom stage cannot absorb. The gap is not the meter's model: the meter is calibrated AGAINST the sim as a research instrument and is wired into no gate — not e2e, not `selection_warning_count`, not `validate/`. A fast below-plan predictor for the whole deep-chain class exists and is unused. Remaining caveat: the original 68.2% run converged at the *minimum* checkpoint count and is unreconciled with #591's PR note claiming 90–98%. Full entry + repro: `rfc064-phase2-followups.md` §9.
    - **CALIBRATED ON POST-LIFT LAYOUTS 2026-08-08, and the FLOOR PROPERTY DID NOT SURVIVE.** The corpus result that made a meter-backed gate look safe — "meter says below plan ⇒ believe it", dangerous quadrant empty across 41 rows at every tolerance ≥90% — is a property of **pre-lift** layouts. Measured on six post-lift layouts (`crates/meter/examples/sweep_postlift.rs`, bank at `~/spaghettio-corpora/postlift-2026-08-07/`, all runs warmup 432k / converged / kit-clean): the meter reads **96.0% on `tier2-ec10-lift` where the sim delivers 89.7%**, a **missed defect at 95% on both metrics** (the 90% row also flags but is knife-edge — the sim's own produced/delivered columns straddle that cutoff by 1.2pp, so only the 95% classification is metric-stable), and optimism reaches **4.3× the corpus maximum** like-for-like (+5.60% sim-relative on produced against the corpus's +1.3%; +7.03% on delivered). So a report-only gate is still shippable (a gate that misses blocks nobody) but **must not carry the floor claim**. Separately, `pu1-lift` reads 77.81% against a sim delivering 102.01% (**−22.71% sim-relative on produced, 1.7× the corpus worst**; −23.73% on delivered) — a *false accusation*, root-caused to petroleum-gas distribution inside the meter's own fluid network, **not** the research-productivity axis (that fixture declares it, kit-clean). The two are independent: fixing the petroleum defect removes the false accusation and leaves the floor retraction standing. **Caution on strength**: one fixture produces the missed defect, which is enough to remove the warrant but *not* enough to estimate a miss rate — the meter is "not known to be a floor" post-lift, not "fails at 1-in-6". The tier2 optimism was already recorded on 2026-08-07; what is new is its status (classified against gate tolerances on the harness's own verdict metric), so this is a known caveat promoted to a falsification rather than a fresh discovery. Provenance: 5 of the 6 sim runs sit at the harness's *minimum* checkpoint count (class 5c — confirm with a longer warmup); `tier2-ec10-lift`, the load-bearing row, is the exception at 8. Also note the two sweeps report different units (`sweep_corpus` sim-relative %, `sweep_postlift` planned-relative pp); cross-sweep claims must be made in sim-relative terms. Full entry: [`meter-divergence.md`](meter-divergence.md) §2026-08-08; the fluid defect: [`meter-fluid-followups.md`](meter-fluid-followups.md).
- **Belt-detour survey finding (2026-08-01)**: the new `belt-detour` check (`crates/core/src/validate/belt_detour.rs`, `measure_belt_runs`) surveyed 35 tier/stress fixtures (5543 belt runs; `scratchpad` artifact not committed — see the check's PR for the summary) and found the corpus is overwhelmingly clean (99th-percentile run is ~1-2 tiles of excess) except two reproducible pathologies: (1) `advanced-circuit` layouts (`Pooled` and `PartitionedDecomposed`, multiple rates, both from-plates and from-ore) route their last row segment 2.0-6.25x its endpoint separation, 9-21 excess tiles — 9 of the corpus's 5543 runs, all in this one recipe family; and (2) the RFC-052 **mega-chain composition** path (`bus::cells::chain::compose_chain`) ships belt runs 2.6-3.0x their endpoints' Manhattan separation, with 44-137 excess tiles, on both `chemical-science-pack@5` and `processing-unit@4` from-raw fixtures (`cell_composition.rs`'s `mega_chain_chem5_resolves_bus_failure` / `mega_chain_pu4_resolves_bus_failure` permanent gates — both, plus five affected e2e fixtures, now tolerate `belt-detour` explicitly rather than silently, per their updated comments). Neither is root-caused or fixed; flagged here as open layout-quality gaps (the rest of the corpus is clean). Also worth recording: `belt-detour` is deliberately excluded from `validate::selection_warning_count` — including a brand-new, uncalibrated-for-selection category by default flipped several fixtures' `decomposition_search` candidate choice on first wiring, with no sim evidence either candidate was actually worse. (2026-08-14, #632 B6: the exclusion set is now the three-category `SELECTION_EXCLUDED_WARNING_CATEGORIES` const — `belt-detour` plus the demoted `inserter-throughput` / `inserter-item-throughput`; `row-output-lane-budget` was briefly demoted with them and reinstated the same day when its trust-table row was corrected to threshold-sim-calibrated (#431); `input-rate-delivery` was lifted INTO selection 2026-08-07 and stays in.)

(Audited 2026-07-21: #65, #68, #136, #310 — previously cited here — are all
closed. 2026-07-24: #334 closed — the (7,3)/(7,4) lane skew is ACCEPTED as a
documented limitation (user call), guarded by `balancer_lane_audit`'s
KNOWN_IMBALANCED tripwire; #266's (5,8) MX1 limit accepted the same way,
guarded in `balancer_classify`. Both revocable on re-bake or field failure.
2026-08-14: the #334 acceptance is REVOKED BY DELETION — (7,3)/(7,4) were
culled from the library with the 12 waist-capped shapes (#632 A3;
corpus-unexercised per `scripts/balancer_usage_census.py`), KNOWN_IMBALANCED
is empty, and the min-cut ≥ rated invariant holds library-wide — enforced
in CI since the same day by `balancer_lane_audit::audit_min_cut_capacity`,
which superseded and deleted the `scripts/balancer_cut_census.py` instrument
that first measured it (RFC-027's decision log has the full adjudication).
2026-07-31, 28-issue audit: #513 and #429 closed as shipped — RFC-060 and
RFC-053 respectively; #429's geometry residue filed as #526. #311/#312/
#335/#336/#345 closed not-planned into parked cluster #527, revisit at the
cell-interface RFC.)

## Deferred tooling tasks

- **Meter-vs-Factorio calibration matrix (active, 2026-08-22):** the current
  35-fixture e2e corpus now has one shared definition, a tracked exporter, an
  immutable Factorio-runner, and a meter reader that reports coverage as
  `measured/expected` rather than silently narrowing its denominator. The
  first full current-generation bank is being measured; its reports are not
  yet a calibration result. Workflow and interpretation:
  [`meter-calibration-matrix.md`](meter-calibration-matrix.md).

Test-suite time recovery (audited 2026-07-19, pick-up notes per item in
[`test-suite-followups.md`](test-suite-followups.md)): committed STRESSGOLD
baseline goldens landed 2026-07-19 and were DELETED 2026-08-15 (#632 B7 —
host-cache-relative so never CI-enforceable, unrun for three weeks, and
their only consultation produced false drift signals; the `STRESSGOLD`
hash-print protocol survives, see the followups entry); CI nextest parallelism re-enable via
timeout-ceiling bumps (~5 min/push, experiment already documented in
`.config/nextest.toml`); `[profile.test]` opt experiment for SAT/A*-heavy
tests (measure before adopting).

**Clippy's test/example debt: measured 2026-08-06 — the deferral never sized
it; it is 28 sites.** Not a newly-found gap — `ci.yml`'s clippy step carries
the comment *"Lib-only on purpose: `--all-targets` trips pre-existing
test/example debt"*, a deliberate deferral taken at the #434
workspace-widening on 2026-07-24 (commit `b31c7bb0`, which wrote that comment
in the same change). What is new here is the size and the proof that clearing
it is behaviour-free.

**28 warning sites (14 distinct file × lint pairs)** live in
`crates/core/tests/*` and in `#[cfg(test)]` modules under `src/` — invisible
to every gate we run. Measured by inventorying warnings *without* `-D`: with
it, cargo aborts at the first failing unit, so a plain run reports a truncated
subset, and two such runs stop at different points and are not comparable to
each other. Identical inventories on `origin/main` `dbeed392` and on the
RFC-064 productivity stack tip, `comm` empty in both directions —
independently re-derived by a second agent via a different method
(`--message-format=json`, dedup by file+line+lint). Pure backlog, nobody's
regression.

Composition: 6 `type_complexity` + 3 `too_many_arguments` (e2e helper
signatures — type aliases or a test-scoped `#[allow]`), 5
`doc_lazy_continuation`, 4 `field_reassign_with_default`, 2 `dead_code`, 2
`unnecessary_sort_by`, and 6 singletons. Nothing architectural; the whole set
is mechanical and behaviour-free. Three of the sites are #582's leavings — the
unused `DICoupling` import at `objective.rs:486` (one of the singletons) and
the dead `belt`/`inserter_at` helpers at 871/875 (both `dead_code` entries) —
written by #569 and orphaned when #582 deleted the duplicate §(b)
implementation.

**One more gate shares the blind spot**, so the flag has to move in two places
at once: `.githooks/pre-commit` runs `cargo clippy -p spaghettio_core` —
core-only *and* lib-only, i.e. narrower than CI, so a hook-green commit can
still fail CI. And one more **target** starts being covered when it moves:
`crates/core/examples/sim_export.rs`, load-bearing wiring since #591, is
unlinted today for the same reason (it contributes zero warnings, so it costs
nothing to bring in).

**Recommended: clear it in one mechanical pass, then flip `--all-targets` in
`ci.yml` *and* the pre-commit hook.** An earlier draft here said "clean
opportunistically as PRs touch those files"; the evidence in this very
paragraph argues against it. That strategy has already been run and failed —
`e2e.rs` and `layout.rs` are among the most-touched files in the repo and
accumulated anyway — and the class regrows under the current regime, with the
3 `objective.rs` sites above arriving in the week to 2026-08-06 out of
careful, reviewed work. The
backlog is 1-2 hours and cannot regress anything CI-visible, because CI cannot
see any of it. The flag is the fix; the cleanup is a one-time toll, not a
programme.

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
