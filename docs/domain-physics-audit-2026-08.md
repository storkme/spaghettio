# Domain-physics factoring audit — 2026-08-20

**Status**: point-in-time audit (companion to
[`offpath-code-followups.md`](offpath-code-followups.md)); its two live
findings are tracked as B1/B2 on the campaign issue #675. Line numbers are
as of the audit date — verify before citing. Method: for each of nine
mechanic families, every site that *encodes* the mechanic (implements its
logic or hardcodes its constants, not merely calls a shared helper) was
enumerated across `crates/core`, `crates/meter`, and tooling; each family
got a verdict — SINGLE-SOURCE / DUPLICATED-CONSISTENT /
DUPLICATED-DIVERGENT / UNCLEAR. `crates/sim-harness` is ground truth, not
a model, and is excluded from duplication counts. The two
DUPLICATED-DIVERGENT live findings were re-verified at source in-session
before being reported; finding 1 was subsequently adjudicated against
game truth and its attribution INVERTED — see its entry.

## Headline

**The domain layer is real and mostly single-sourced.** `common.rs` (belt
tiers, UG reach, entity size, pole coverage/reach, inserter
reach/hand/lane), `fluid_ports.rs` (port geometry), `recipe_db.rs` +
`module_policy.rs` (crafting/module math), and `balancer_classify.rs`
(splitter composition ground truth) each own one family and are consulted
by generation, validation, and CLI tooling alike. The meter's
re-derivations are *deliberate* verification architecture — KC4's
`kc4_independence.rs` test forbids it importing `spaghettio_core` — and
are mostly guarded by cross-checks or documented-gap comments
([`meter-divergence.md`](meter-divergence.md)). The codebase's mess is not
in its domain modelling; it was one level up, in the experiment layer the
off-path deletion campaign drained.

## Family verdicts

| # | Family | Verdict | Authority |
|---|---|---|---|
| 1 | Belt lane mechanics | **DUPLICATED-DIVERGENT** (meter; see finding 1) | `belt_flow.rs::lane_transfer` is the richest model; `common.rs` owns lane tables |
| 2 | UG pairing + reach | DUPLICATED-CONSISTENT (+ latent turbo asymmetry, finding 3) | `common.rs::ug_max_reach` |
| 3 | Splitter behavior | SINGLE-SOURCE, with a *deliberate* writer/reader adversarial pair | `balancer_classify.rs` (composition); `ghost_router.rs`/`belt_structural.rs` (priority pair) |
| 4 | Belt throughput + stacking | SINGLE-SOURCE, zero stray literals | `common.rs::BELT_TIERS` + `*_stacked` |
| 5 | Inserter mechanics | SINGLE-SOURCE in core; meter diverges deliberately (governed, documented) | `common.rs` I3/I4/I5/I6/I8 tables; meter has the only swing-timing model in the workspace |
| 6 | Fluid mechanics | SINGLE-SOURCE per layer (+ meter mirror-flag risk, finding 2b) | `fluid_ports.rs`, `validate/fluids.rs`; F13 prevented in `netflow.rs`, executed only in `meter/machine.rs` |
| 7 | Entity footprints | SINGLE-SOURCE; parser's second table pinned by a parity test | `common.rs::entity_size`/`oriented_entity_dims` |
| 8 | Power coverage/wires | SINGLE-SOURCE; doc comments name the past duplication bugs unification fixed | `common.rs::supply_area_distance`/`pole_wire_reach` |
| 9 | Crafting/module math | SINGLE-SOURCE except **one live bug** (finding 2a) | `recipe_db.rs` + `common.rs::module_effect` + `module_policy.rs` |

## Findings (ranked)

1. **Curve-chirality divergence — ADJUDICATED 2026-08-21, and the
   original attribution here was INVERTED.** The audit accused the
   meter's `LaneMap` (no swap variant, every curve index-preserving) of
   diverging from `belt_flow`'s chirality-dependent swap. Adjudication
   against game truth (B11, expert-confirmed: lane contents never jump
   lanes through a turn, either chirality) plus both models' seeding
   conventions (BOTH are handed — meter via `left_of`/`near_lane_from`,
   belt_flow via `LANE_LEFT` → index 0) shows identity-on-curves is
   CORRECT and the swap was the bug: **the meter is cleared;
   `belt_flow::lane_transfer`'s cross-product swap contradicted B11 on
   one chirality**, invisible on the corpus because symmetric lane rates
   make a swap a no-op. Fixed with a both-chirality discrimination test
   (failed exactly [0,5]-vs-[5,0] on N→E pre-fix). A model-comparison
   lesson: divergence identifies a disagreement, not the guilty side —
   adjudicate against game truth before attributing.
2. **Two more divergences**: (a) `analysis.rs`'s module speed/prod
   aggregation is quality-blind — it never reads the `ModuleItem::quality`
   field `module_policy.rs` sets and scales by, so `blueprint-analyze`
   underestimates quality-module blueprints (legendary speed module: +50%
   vs the planner's +125%; **verified at source** — B1 on #675; analysis
   tool only, generated layouts unaffected). (b) `meter/factory.rs`
   fakes fluid-port mirroring by treating oil-refinery/foundry/cryo as
   always-reversed — self-documented as mis-binding a genuinely-unmirrored
   instance (B2).
3. **Latent tier asymmetry** (family 2): the meter models a Turbo belt
   tier (UG gap 10); core's `ug_max_reach` falls through `_ => 4` — one
   recipe-data change from a silent misreach (B2 guard).
4. **Three-deep lane-classification redundancy in core** (family 1):
   `belt_flow` (lane-index math), `connectivity` (coarser topology),
   `ghost_router` (generation-side). Consistent today, but it is the
   single most failure-prone mechanic in the repo's history and nothing
   pins the three together the way the splitter pair is pinned.
5. **Pairing *logic* vs pairing *constants*** (family 2): the UG reach
   constant is single-sourced but the pairing search is re-implemented at
   each call site (eviction, ghost_router, belt_flow — the canonical
   derivation is `connectivity::build_ug_pairs`, which the UG checks now
   consult; residual copies are consistent).

## Good patterns worth preserving (found, not prescribed)

- **Unification receipts in doc comments**: `supply_area_distance`,
  `pole_wire_reach`, and `entity_size` each *name the historical
  duplication bug* their unification fixed. The gold standard here.
- **The deliberate adversarial pair**: splitter priority is implemented
  independently by the generation writer (`ghost_router`) and the
  validation reader (`belt_structural`), kept in sync by a fixture —
  intentional cross-checking, not drift.
- **Parity-pinned second tables**: `blueprint_parser.rs`'s entity-size
  table (needed to recognize imported entities) is pinned against
  `common::entity_size` by an automated drift test; the meter's
  productivity formula is pinned against the engine's by
  `productivity_matches_engine_formula`.
- **KC4 isolation as architecture**: the meter cannot import core, so it
  can catch the engine's own arithmetic mistakes; divergences are
  *tracked* (`meter-divergence.md`) rather than forbidden.

## Rule-ID citation discipline (factorio-mechanics.md)

Strong: splitters (S-series), belts/UG (B/U-series), inserters
(I-series). Absent: fluids (zero F-series citations — code cites RFC
names/issues instead), power/footprints (no assigned IDs), modules (zero
MB-series citations). Not itself a defect; recorded so a future
mechanics-doc pass knows where the cross-referencing stops.

## Corrections this audit produced elsewhere

- The team memory claiming "belt stacking is the unmodeled mechanic" was
  stale (belt-stack throughput is fully modeled since RFC-046); the
  genuinely unmodeled piece is inserter stack *pickup*. Memory corrected
  2026-08-20.
