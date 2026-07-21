# Project status ledger

**Status (2026-07-21)**: moved out of `CLAUDE.md` so the agent-context file
sticks to process; this is the canonical home for capability status now.
Update this file (not `CLAUDE.md`) when a tier's status changes or an RFC
closes out. Per-topic backlogs stay in their own `*-followups.md` docs; this
ledger is the cross-cutting view.

## Recipe complexity ladder

Tracks which recipes produce zero-error bus blueprints. Moving up = real progress. Tests for each tier live in `crates/core/tests/e2e.rs`.

| Tier | Recipe | Complexity | Bus status |
|------|--------|-----------|-----|
| 1 | `iron-gear-wheel` | 1 recipe, 1 solid input | SOLVED |
| 2 | `electronic-circuit` | 2 recipes, 2 solid inputs | SOLVED (incl. from ores) |
| 3 | `plastic-bar` | 1 recipe, 1 fluid + 1 solid input | SOLVED |
| 4 | `advanced-circuit` | 5+ recipes, mixed solid/fluid | SOLVED (`tier4_advanced_circuit_from_ore_am2` green: AC@5/s ores AM2 yellow, 0 errors). Carries 1 input-rate-delivery (unrelated, pre-existing demand-pull modeling residual); inserter-item-throughput 0 since the last-in-row belt extension (`0d7132c`, 2026-07-19; was 4, and 58 masked sides pre-`rfc-inserter-sizing.md`). From plates still has lane-throughput warnings, [#65](https://github.com/storkme/spaghettio/issues/65). |
| 5 | `processing-unit` | Deep chain, multiple fluids | SOLVED (`tier5_processing_unit_from_ore_am3` green: PU@2/s ores AM3 red, 0 errors, Pooled — fully clean, 0 warnings, since the last-in-row belt extension `0d7132c` 2026-07-19; was 5 inserter-item-throughput, and 129 masked sides pre-`rfc-inserter-sizing.md`). Higher rates / partitioned strategies still have junction + starvation issues — see `partition_strategy_scoreboard_extended`. |
| 6 | `flying-robot-frame` | Adds lubricant: advanced-oil-processing refinery rows with 3 fluid outputs | SOLVED via the USP chain (0 errors). The 2026-07-11 "0 warnings" reading predates the per-item inserter-attribution check landing — see tier 7 and the corpus-wide note below. No dedicated FRF fixture yet. |
| 7 | `utility-science-pack` | Very deep chain (LDS + PU + FRF) | SOLVED (`science_gauntlet` USP@1/s AM3: 0 errors, 6615 entities, 208×281). Utility itself fully clean since the last-in-row belt extension (`0d7132c`, 2026-07-19; was 2 inserter-item-throughput). Across the six packs the only residual is production-science: 8 inserter-item-throughput, likely the same last-in-row trim still present in the triple/quad/hstack templates (follow-up). Logistic/military science packs clean at 1/s (previously carried input-rate-delivery residue, since fixed). |

## Recent RFC close-outs

**`rfc-inserter-sizing.md` close-out (2026-07-13)**: bus inserters are now sized to planned per-machine throughput via a shared regular→fast→stack ladder (long-handed count-ladder for reach-2 sides), with an ingredient-to-belt reassignment lever and a user-facing `max_inserter_tier` engine param (wasm-bindings + web UI, URL-encoded). `science_gauntlet` 1/s inserter-throughput/item-throughput warnings across the six packs: **140 → 12** at close-out (automation/logistic/military fully clean; chemical 1, production 9, utility 2 residual, all under the newer, stricter per-item check — the old aggregate check is at 0 everywhere), then **12 → 8** after the 2026-07-19 last-in-row belt extension (`0d7132c`: chemical and utility now clean; all 8 remaining are production-science). The "untouched triple/quad/hstack trims" hypothesis for those 8 was **falsified 2026-07-20** (`acd147e` extended the pattern to triple_input_row — quad/hstack are structurally immune — and the 8 turned out to be 6 input3 contest-losses + 2 genuine far-side rate walls; see [`inserter-throughput-followups.md`](inserter-throughput-followups.md)). This is **validator-verified only** — the RFC's two in-game blueprint-import anchors (kill criterion 5) remain open until the user runs them; see the decision log in [`rfc-inserter-sizing.md`](rfc-inserter-sizing.md) for the full phase-by-phase evidence trail.

**`rfc-build-quality.md` close-out (2026-07-20)**: user-facing **build quality** param (normal→legendary, `quality`/`q=` URL-encoded through wasm `solve`+`layout` and the sidebar). Solver machine counts scale ×(1+0.3·level) via `effective_crafting_speed`; the inserter ladder, pole supply radii (+1/level), and wire reach (+2/level, shared table `common::pole_wire_reach` consumed by placement, the emitted `wires` artifact, and the validator) are quality-aware; functional entities (machines/inserters/poles — never logistics) get `PlacedEntity.quality` stamped in one `layout_pass` post-pass, validators rate each entity by its own tier, and export emits the lua-api `quality` field (parser reads it too, so imported quality blueprints validate). Normal is bit-identical to pre-RFC (kill-criterion-2 gates: unit bit-equality sweeps + full suite + STRESSGOLD check). The 60 EC/s legendary headline is capped at 45/s (one blue belt) until [#311 output-merger capacity](https://github.com/storkme/spaghettio/issues/311) closes; [#312](https://github.com/storkme/spaghettio/issues/312) tracks the quality-magnified consumer-clamped fan-in wall; [#310 pole-band thinning](https://github.com/storkme/spaghettio/issues/310) is the designated next pick-up. **In-game import anchor still open** (user-run; unblocked — #313 resolved as premise-falsified: the engine's `stack-inserter` IS the current Space Age stacking inserter). Full trail: [`rfc-build-quality.md`](rfc-build-quality.md) decision log; renderer constraints learned en route: `web/CLAUDE.md`.

## Open tracking issues (layout quality)

[#135 balancer templates are oversized](https://github.com/storkme/spaghettio/issues/135), [#136 missing coprime balancer shapes](https://github.com/storkme/spaghettio/issues/136), [#68 fluid row 3-tile pitch](https://github.com/storkme/spaghettio/issues/68) (design: [`rfc-fluid-dual-input-row.md`](rfc-fluid-dual-input-row.md)).

## Deferred tooling tasks

Test-suite time recovery (audited 2026-07-19, pick-up notes per item in [`test-suite-followups.md`](test-suite-followups.md)): committed STRESSGOLD baseline goldens landed 2026-07-19 (`SPAGHETTIO_STRESS_GOLDEN=check|bless`, see `crates/core/tests/goldens/stress/README.md` — host-cache-relative, opt-in, not CI-enforced); CI nextest parallelism re-enable via timeout-ceiling bumps (~5 min/push, experiment already documented in `.config/nextest.toml`); `[profile.test]` opt experiment for SAT/A*-heavy tests (measure before adopting).
