# Off-path code followups — the golden-path deletion backlog

**Status (2026-08-20, end of day)**: audit COMPLETE; execution underway —
tracking issue **#675** is the live queue. Tier 1 item 1 (row_rotation)
MERGED (#670); items 2–4 MERGED (#674); Tier 2 item 5 (bands) MERGED
(#676); item 7 (CP-SAT, grown to a seven-file cluster in execution)
MERGED (#677). Owner
decisions taken 2026-08-20: Tier 2 items 5–8 all APPROVED for deletion;
item 9 (inserter-check pair) still open; Tier 3 resolved KEEP-CAMPAIGN
(RFC-068 P0 ran and PASSED, #672). Tier 1 below is
actionable by any session without further evidence. Tiers 2–3 each name the
single owner decision that unlocks them. Bugs and stale-docs findings at the
bottom are independent of any deletion.

## What this tracks

A coverage-driven audit of `crates/core/src` answering: *which code does the
engine actually execute to produce the layouts we ship, and what is everything
else for?* Motivation: progress has slowed while the engine accreted
experiment residue; every live path is claim surface that each change must be
re-verified against (see `docs/pr-churn-audit-2026-08.md` for the measured
mechanism).

## Method + reproduction

Three instrumented runs over `crates/core` (cargo-llvm-cov 0.9.0, toolchain
1.95.0, `SPAGHETTIO_ZONE_CACHE_PATH` pinned to `sat-zones-ci.bin` per the
measurement protocol; pin file restored afterwards):

- **Run A ("golden")** — the known-good fixture corpus: all 76 non-ignored
  e2e tests minus the 3 refusal tests (`stacking_refuses_low_inserter_cap`,
  `di_jammed_cell_is_visible_and_therefore_refused`,
  `di_cell_output_belt_exemption_does_not_cover_the_consumer`), plus the
  `#[ignore]`d six-pack `science_gauntlet` (one test, six packs at 1/s), plus
  the non-ignored `mega_*` composition gates in `cell_composition.rs`. All
  green: 73 + 1 + 5 = 79 tests (76 non-ignored per libtest's own `--list`;
  a source-attribute count will disagree — the listing is authoritative).
- **Run B (full default suite)** — everything CI gates. Caveat hit and
  repaired: cargo's fail-fast stopped the run when
  `tier2_electronic_circuit_20s_from_ore` overran its hard 10s wall guard by
  4ms under instrumentation, silently skipping 11 test targets; a
  `--no-fail-fast` repair pass (B2) unioned them back in. 152 functions moved
  dark→edge-only in the repair — treat any future single-run coverage sweep
  with a mid-suite failure as incomplete by construction.
- **Run C (cold-cache probe)** — two from-ore fixtures with an empty zone
  cache to capture live SAT solving. Added nothing beyond A (the golden runs
  already exercise it).

Function classes (v0-mangling deduped across the lib and its unit-test
compilation — without that dedup every function double-counts):

| Class | Meaning | Production fns | ~lines (region span) |
|---|---|---|---|
| golden | executed producing shipped layouts | 1,866 (64.4%) | — |
| edge-only | reached only by unit/audit/diagnostic tests | 637 (22.0%) | ~10,081 |
| dark | reached by nothing in the suite | 393 (13.6%) | ~4,243 |

**Coverage is not liveness.** Verdicts below additionally rest on
production-consumer checks (wasm-bindings, mining-cli, sim-harness,
balancer-gen, scripts/) and per-module RFC ownership. Confirmed traps that
pure coverage would have mis-called: `region_reimprove` (dark in tests, but
auto-fires in the web app via `improve_region_streaming`), `fixture.rs`
(backs the in-canvas SAT-zone editor), `tapoff_search.rs` (entirely
`#[cfg(test)]` — a brute-force *proof* that the hardcoded 2×2 tap-off stamp
is optimal, invisible to production coverage by construction), and the whole
offline balancer-generation toolchain.

## Tier 1 — deletable now, no owner call (~2.7k lines, ~2.9k with in-file test blocks)

"Zero callers" throughout this tier means zero *production* callers — several
items have `#[cfg(test)]` callers in their own file (the belt_flow twins'
test blocks ~100 lines, `partitioner.rs` helper tests ~5,
`analyze_blueprint_string`'s 2 tests, ~15 sat.rs tests to retarget); delete
or retarget those in the same change, they are counted in the 2.9k figure.

1. **`bus/row_rotation.rs`** (~2,328 lines with wiring): the ad-hoc RFC-064
   rotation-aware rigid-row prototype, RETRACTED in RFC-064's own decision
   log 2026-08-03 (exhaustive search on frozen sci2-ore: zero admissible
   candidates). Zero production consumers; entry points documented in
   `bus/layout.rs:339-343` as deliberately disconnected from selection. Scope:
   the file + `layout.rs` entry points (`build_rotation_aware_row_layout*`)
   + `bus/mod.rs` decl + its tests in `rfc064_packed_router.rs` (~54 lines);
   update (don't delete) comment refs at `cells/mega.rs:658`,
   `cells/chain.rs:1734`. Precedent: #632 A2 deleted the sibling retracted
   spike (`bus::compaction`) outright.
2. **Orphaned validator twins in `validate/belt_flow.rs`** (~191 lines):
   `check_belt_throughput` (:1375), `check_output_belt_coverage` (:1418 —
   also *stale*: missing the Fulgora recycler-eject handling the live twin
   has), `check_belt_inserter_conflict` (:1764). `validate/mod.rs` dispatches
   only the `belt_structural::` copies (mod.rs:1096/1097/1143); zero callers.
   Leftovers of the era the #632 B5 twin-arbitration partially cleaned.
3. **`bus/junction_solver.rs` dead cluster** (~127 lines): `GrowingRegion::
   {grow, recompute_bbox, promote_fully_enclosed}`, `bbox_tiles_set` — six
   `#[allow(dead_code)]` markers, superseded by `expand_bbox`.
4. **Misc verified-dead** (~80 lines): `bus/partitioner.rs` helpers
   (:75/:93/:106/:167/:175/:984 — `plan_partitioning` re-derives the
   utilization formula inline instead of calling the dead helper);
   `sat.rs:1747/1768` legacy `solve_crossing_zone{,_with_stats}` wrappers
   (production is `_per_channel`; ~15 sat tests need retargeting);
   `zone_cache.rs:116` `cache_stats` (superseded by `_extended`);
   `snapshot.rs:172` `read_from_file` (zero callers; the documented read-back
   is the shell pipeline); `analysis.rs:649` `analyze_blueprint_string`
   (mining-cli calls `_any` exclusively); `junction_sat_strategy.rs:256`
   `impl Default for SatConstraints`. Two items adjudicated no-change at
   execution (#674): `ghost_router.rs:262`'s dead arm stays (exhaustiveness-
   load-bearing; `unreachable!()` would trade a silent no-op for a WASM
   panic on a wrong analysis) and `recipe_db::machine_for_recipe` stays
   (4-line doc-anchor wrapper, 10+ test callers — deletion is
   churn-positive).

Tier 1 items are still **layout-engine-adjacent deletions**: run the full
verification protocol (suite green + clippy + WASM build), not just compile.

## Tier 2 — one owner decision each (~8.4k code lines + 20.8k data)

5. **`bus/bands.rs`** (~4,400 lines: module 1,812 + `rfc064_packed_router.rs`
   255 + the RFC-058 block in `cell_composition.rs` ~2,174 + `layout.rs`
   wiring ~150 + trace variants ~35). RFC-058's falsification record, kept
   "in-tree, default-off, as the reproducible record" — but all three named
   future consumers (RFC-058, RFC-063 Phase C, RFC-064 Phase 3) have since
   run and concluded/failed. Same shape as what #632 A2 deleted; it just
   wasn't named. **Decision TAKEN 2026-08-20: yes — DELETED, #676.**
6. **`bus/cells/placement.rs` + `ChainOrder::Compact`** — **DELETED
   2026-08-20 (#678)** (~1,000 lines):
   RFC-057 leftover the A2 sweep missed; `Compact` is a private enum variant
   constructed only by `compose_chain_compact`, called only by 3 `#[ignore]`d
   RFC-055 tests. Deletion requires surgery in `cells/chain.rs` (heavily
   golden-covered) — full protocol.
7. **The dead "canonical" CP-SAT pipeline** (~2,765 lines):
   `balancer/placement/cp_sat.rs` (287) + `tests/cp_sat_round_trip.rs` (167,
   env-gated, zero CI refs) + `scripts/cp_sat_placer.py` (1,762) +
   `scripts/bake_cp_sat_runner.py` (549). RFC-023 ended at "wire in via a
   separate RFC" (2026-05-01); that RFC never happened; untouched 3+ months
   while the supposedly non-canonical `balancer-gen` path was actively
   extended. Bundle: drop the `placement/` re-exports, and consider inlining
   `PlacementEngine`/`library_lookup` into `balancer_engine_bench.rs` (the
   trait exists only for the CP-SAT plug-in that never landed). **EXECUTED 2026-08-20 (#677)**, cluster grown to seven files
   (bake_cp_sat_report.py, bake_overnight.sh, bake-overnight-results.md
   found by reference sweeps). Including the fix done with it:
   `crates/balancer-gen/scripts/place.py`'s
   docstring still names the dead script "canonical" and itself "NOT
   canonical" — inverted reality.
8. **`LayoutStyle::Spaghetti` arms** (~470+ lines): every production caller
   passes `Bus`; `Spaghetti` is constructed only in tests yet is the derived
   *default* (a standing footgun, trust-table hole 4). Biggest single piece:
   `belt_flow.rs:675` `check_belt_network_topology` (458-line cluster),
   gated `== Spaghetti`, dead in production; two more same-gated severity
   branches hide inside otherwise-live functions (`belt_flow.rs:533`,
   `:1121`) and go with it. Deleting the enum + severity forks simplifies
   `validate()`'s signature. **Decision TAKEN 2026-08-20: relic — deleted (T2d).**
9. **`inserter-throughput` + `inserter-item-throughput` checks** —
   **DELETED 2026-08-21 (owner decision + T2e PR)** (validator):
   demoted from selection 2026-08-14 (#632 B6), hand-capacity model never
   sim-anchored, now report-only. Cost of deletion: the only signal on
   production-science's known 8-warning residual, and the big-electric-pole
   canary (whose real protection is RFC-059's teeth test per the trust
   table). **Decision: accept unreported residual until a calibrated model
   exists?** General validator note: quiet ≠ ditchable — structural checks
   refuse Error-carrying candidates *inside* selection, and no census of
   per-category firing across candidate evaluations exists yet (instrument
   gap below). No other check deletions are recommended.

## Tier 3 — governed by one question: is RFC-068 alive? (~3.7k lines + 20.8k data)

`bus/transit.rs` (1,207), `bus/candidate_runner.rs` (369 + 437 tests),
`objective.rs` (1,005 + 104 tests), `verdict.rs` (689),
`bus/template_candidate.rs` (200 + 149 tests) are all kept alive, by name, in
RFC-064's 2026-08-14 decision log *solely* as the RFC-068 celldb campaign's
entry path (`run_candidate_field` → `objective::measure` → Transit scoring).
**RESOLVED 2026-08-20: RFC-068 is ALIVE — Tier 3 is KEEP-CAMPAIGN.** The
owner's status check ran P0 the same day; it PASSED (K68-1, full verdict
parity, zero escape hatches — RFC-068 decision log). The five modules
stay, now with a live consumer again; this tier re-opens only if a later
kill criterion (K68-2 donor half, K68-3) fires. Original stall evidence,
kept for the record: PR #628 (docs-only) merged 2026-08-13, P0 not
started, tracking #629 silent for 7 days. `celldb.rs` (617) + `data/celldb.json` (20,775 lines)
additionally carry RFC-067's independent "measured baseline" retention and
survive even if RFC-068 dies; `preview.rs` likewise (K67-2 killed its
consumer but the RFC forbids rebuilding without a decision-log amendment and
retains the module as baseline). **Decision: owner status check on RFC-068
before touching any of this.**

## Explicitly cleared — do not re-litigate without new evidence

`tapoff_search.rs` (cfg(test) proof-test); `template_validate.rs` (shipped
gate in balancer-gen's bake path, used 2026-08-14); `cells/compose.rs`
(`stamp_path` is an unconditional chain-router dependency); `cells/chain.rs`
+ `cells/mega.rs` (golden); `region_reimprove.rs`, `fixture.rs`,
`preview.rs` (prod/web/RFC-contract); `bin/import_balancer.rs` + the
`balancer/{graph,synth,verify,bake}` subtree + `balancer_topology.rs`
(live offline library toolchain, last run 2026-08-14); `short_ids.rs`
(zero non-test Rust callers by design — it exists to keep the committed
`short-ids.json` honest for the web app's independent TS port); `balancer_generate.rs`
(`merge_tree` is runtime-load-bearing despite the file name); the
`ModuleSizeSplit`/`k1-shape-fix`/`merge-tap` candidate machinery in
`decomposition_search.rs`+`partitioner.rs` (~520 lines: user-reachable via
`strategy=partitioned-decomposed`, though provably untriggered on the current
corpus — a *policy* question about supporting that strategy axis, not dead
code); netflow's `allow_voiding` branch (parked pending UI hookup).

## Bugs found en route (each independently actionable)

- **`analysis.rs:368-372` is quality-blind** (VERIFIED at source): module
  speed/prod aggregation never reads `ModuleItem::quality`, diverging from
  `module_policy.rs:148-152`'s quality-scaled formula — `blueprint-analyze`
  silently underestimates any quality-module blueprint (legendary speed
  module: +50% vs the planner's +125%). Affects the analysis tool only, not
  generated layouts.
- **Curve chirality — ADJUDICATED 2026-08-21, attribution INVERTED**
  (see `domain-physics-audit-2026-08.md` finding 1 and #683): the meter
  was accused and CLEARED — its identity-on-curves matches game rule
  B11; the chirality-dependent swap lived in `belt_flow::lane_transfer`
  and was the bug (fixed #683, one fabricated ac-am2-ore warning
  re-blessed with position forensics). Do NOT "fix" the meter toward a
  swap; both models now agree with the game and both are test-locked.
- **Meter mirror-flag blindness** (self-documented, `meter/factory.rs:192-210`):
  refinery/foundry/cryo ports assumed always-mirrored; an unmirrored instance
  would mis-bind.
- **UG-reach tier asymmetry**: meter models Turbo (gap 10); core's
  `ug_max_reach` falls through `_ => 4` — one data change from silent
  misreach.
- **Perpendicular-template rung has zero test coverage anywhere** (~795
  lines, `ghost_router.rs:5396-6010`): rung 1 of the live routing-strategy
  ladder, fires on narrow preconditions, and no test in the suite reaches it
  (its sibling `cluster_adjacent_crossings` has nine). Coverage gap, not
  deletion. (Its replay-only factory `perpendicular_template_strategy()`
  was folded into the shared `pinned_tier_core_strategies()` helper by
  #687.)
  - **CLOSED 2026-08-21 (#687), with findings.** Two fixtures now cover
    the rung's logic and the ladder's dispatch via new
    `expected.solved_by` strategy attribution in the region-fixture
    harness (`perp_template_pipe_belt_bridge` = static unit pin of the
    rung's internal pipe×belt logic — NOT a production-reachability
    probe; `perp_template_single_tile_crossing` = the pinned-tier core
    ladder's dispatch on a belt×belt crossing). The instrumentation surfaced:
    1. **The rung appears production-unreachable on BOTH shapes it
       handles.** Belt×belt two-item crossings: `junction_solver`'s
       item-conflict gate skips every strategy on the sole single-tile
       iteration, and the rung refuses any grown region (`tile_count > 1`
       guard). Pipe×belt crossings (#687 round-3 review, verified):
       production dispatch filters pipe specs out of junction seeding
       entirely (`keys_at_tile` — pipes are forbidden tiles; SAT bypasses
       them as obstacles), so the rung's 2-spec predicate refuses, and
       `bridge_belt_over_pipe` never runs from production. The one
       untested reachability hypothesis left: same-item belt crossings,
       *if* those ever seed junctions. Follow-up candidate: a production
       census of junction seeds; if same-item crossings never occur, the
       **entire ~795-line rung** (`solve_perpendicular_template`,
       `try_bridge`, `bridge_belt_over_pipe`, the wrapper) is dead in
       production and deletable — or the conflict/dispatch gates get
       reworked to let the cheap template actually fire. Don't delete on
       this note alone; run the census.
    2. **The fixture replay's strategy ladder had drifted** from
       production (pre-native `sat-1ug`/Relaxed-reach list vs production's
       `sat-1ug-native` core). #687 lifted the pinned-tier core into a
       shared helper (`ghost_router::pinned_tier_core_strategies`) used
       by both production and the replay, so this drift class is closed
       structurally. Auto-tier extras (eviction, AutoUpgrade rungs)
       remain outside the replay — fixtures don't record belt-tier mode
       — so `solved_by` pins are relative to the core only: a change
       confined to auto-mode dispatch is not caught by these fixtures.
- **`region_reimprove.rs` has zero Rust-side tests** while auto-firing in the
  web app on every clean layout with SAT zones.

## Stale docs/comments found (fix independently)

- `docs/rfcs.md` lists RFC-064 as "Design"; it is Active with Phases 0–2
  complete, Phase 3 concluded-failed, P1/P2 code deleted (#632 A2). The
  registry has no status column at all for RFC-020..029, making the dead
  CP-SAT lineage (021–024) indistinguishable from the live one (025–029) —
  the exact mechanism by which Tier-2 item 7 stayed hidden.
- `docs/status.md:184` still names RFC-064 Phase 3 as bands' "next arranged
  consumer"; that phase concluded-failed 2026-08-03.
- `CLAUDE.md` documents one balancer regen workflow; there are three
  pipelines, and the most active (`crates/balancer-gen`) is undocumented.
- `balancer-gen/scripts/place.py` docstring: inverted "canonical" claim (see
  Tier 2 item 7).
- `junction_sat_strategy.rs` header says it wraps `sat::solve_crossing_zone`
  (actually `_per_channel`); `recipe_db::find_recipe_for_item_excluding`'s
  doc cites solver-supplied exclusions deleted by #632 A1; `validate/mod.rs`
  claims `ValidatorSummary` has no in-tree consumer (it's built by
  `blueprint.rs:167/681` for `examples/sim_export.rs`);
  `docs/celldb-phase0-scoreboard.md` is self-declared archivable.

## Instrument gaps (build before further validator/strategy cuts)

1. **Candidate-eval firing census**: per-category issue counts across *all*
   candidate evaluations (not just winners), so "this check never fires" can
   be distinguished from "this check silently vetoes losers". Blocks any
   structural-check deletion beyond Tier 2 item 9.
2. **Meter chirality test** — DONE 2026-08-21 (#685), but note the
   premise inverted first: the test locks the meter's CORRECT identity
   handling (see the adjudicated bullet above), it does not fix a gap.

## Artifacts

The per-file coverage summary and the classified off-path function list
are committed beside this doc
([`offpath-audit-2026-08-20-coverage-summary.txt`](offpath-audit-2026-08-20-coverage-summary.txt),
[`offpath-audit-2026-08-20-offpath-functions.tsv`](offpath-audit-2026-08-20-offpath-functions.tsv)
— gap 3 of the documentation audit, closed 2026-08-20); the raw llvm-cov
JSON stays uncommitted, and the method section above regenerates it from
scratch. Companion audit of domain-physics factoring:
[`domain-physics-audit-2026-08.md`](domain-physics-audit-2026-08.md)
(9 mechanic families, verdicts per encoding site — committed 2026-08-20,
gap 2 of the documentation audit); its two live bugs are the first two
bullets above.
