# RFP: Power placement under face densification

*Revision 2 — rewritten after two-reviewer adversarial review (see decision
log). The largest change from rev 1: Phase 0 is not a mechanical list sync —
it is where the judgment lives (a new energy-source distinction, a sibling
fluid-validation blind spot, and a pole-placement decision for burner
machines).*

## Summary

Poles are placed last (`place_poles`, after row templates) and live off leftover
tiles. Face-densification work — starting with the `beltspan-lastinrow` belt
extension (`0d7132c`), continuing with the face-allocation direction — consumes
exactly those tiles. A corpus census (2026-07-19; 45 snapshots, 2212 poles, 401
rows) shows the risk is **not uniform**: solid rows have real headroom, fluid
rows are already at the edge, and validation has machine-list blind spots — in
*both* the power and fluid checkers. This RFP proposes: **(0)** introduce the
codebase's first machine energy-source fact and unify the four hand-synced
machine lists behind it, closing the power *and* fluid validator blind spots
and deciding pole placement for burner machines; **(1)** replace the
oil-refinery-only pole-gap special case with a principled fluid-row reservation
built on shared (not re-derived) pole-row geometry; **(2)** add a pole-slack
guardrail metric to the stress scoreboard via live decision-time
instrumentation; and **(3)** defer substation/dedicated-power-row design behind
explicit triggers instead of building it speculatively.

## Motivation (census evidence, 2026-07-19; independently audited)

Census: full e2e/stress corpus + the 6 science-gauntlet packs, dumped
snapshots, ground-truth pole analysis. Raw data committed at
`scripts/pole-census-2026-07-19.json`; every statistic below was independently
recomputed from that file during adversarial review and reproduced exactly.
Two reproducibility caveats are recorded at the end of this section.

- **0 power-category validation warnings** across all 45 snapshots today — but
  see the blind spots below before trusting that.
- **82% of poles (1810/2212) sit inside their row's active inserter span** — in
  gaps that face-densification will close. The full classification: 1810
  within-span, 154 beyond-span margin, 136 on fluid faces with no inserters at
  all, 112 unmatched bridge poles (from `repair_pole_connectivity`). The
  within/beyond-span classification is census-defined (its logic lives in the
  census script, not the engine) — counts reproduce exactly, but "beyond-span =
  safe from densification" is an interpretation, not an engine fact.
- **Slack** (free alternative tiles within `place_poles`' own probe window, per
  real pole): solid rows median 8, **zero** poles at 0 alternatives (n=1881).
  Fluid rows median 4, **83/219 (38%) at zero alternatives**. Every zero-slack
  pole in the corpus is on a fluid-involving row.
- **Worst cases, ranked by zero-alternative pole count** (the metric Phase 1
  verification targets; rev 1 mistakenly ranked by median slack, which missed
  the highest-risk fixtures): 10 zero-slack poles (tied worst):
  `issue_136_no_balancer_template_warning_ac5_ore_yellow`,
  `partition_strategy_scoreboard`,
  `stress_advanced_circuit_partitioned_5s_from_plates`,
  `tier4_advanced_circuit_from_ore_am2`; 9:
  `processing_unit_2s_am2_fast_belts_validation_baseline`,
  `tier5_processing_unit_from_ore_am3`; 8: `census_utility_science_pack`.
- **The fluid pole-gap reservation is a special case, not a rule**: the gate at
  `templates.rs:3198` (function at ~3125) fires only for
  `msz==5 && machine=="oil-refinery"`, with no other pole-reservation path
  anywhere in `bus/`. Verified: `RowKind::OilRefinery` is returned both for
  true 5×5 refinery rows (`placer.rs:520`) and for small single-fluid rows like
  chemical-plant lubricant (`placer.rs:533`), but only the former get the gap.
  The reservation costs nothing in inserter terms (0 of 222
  `InserterSideCapped` events correlate with fluid rows).
- **Four hand-synced machine lists, two validator blind spots**:
  - `common::MACHINE_ENTITY_NAMES` (common.rs:13) — canonical, `pub`, 12
    entries, existing drift-regression test at common.rs:431. No wasm-gating
    obstacles.
  - `lane_planner::MACHINE_ENTITIES` (lane_planner.rs:32) — identical 12,
    duplicated "by hand" per its own doc comment; drives pole placement via
    layout.rs:490.
  - `validate::power::MACHINE_ENTITIES` (power.rs:104) — only 6 entries;
    power.rs:139 silently `continue`s past anything else, so coverage is
    **never checked** for `foundry`, `biochamber`, `centrifuge`, `recycler`,
    `cryogenic-plant`, `electromagnetic-plant`.
  - `validate::fluids::MACHINE_ENTITIES` (fluids.rs:21) — 7 entries; the same
    skip pattern (fluids.rs:458) means **fluid-port connectivity is never
    checked** for `foundry` (4 fluid boxes), `cryogenic-plant` (6), or
    `electromagnetic-plant` (4) — real, currently-unchecked ports
    (draftsman-verified). `centrifuge`/`recycler` have 0 fluid boxes and are
    correctly absent. Rev 1 missed this sibling blind spot entirely.
  - The pattern is systemic beyond this RFP's blast radius: review also found
    `analysis.rs:80` `is_crafting_machine()` (CLI-only, deliberately includes
    burner furnaces — no pole-coverage category error) and `common.rs:611`
    `module_slots()` (module counts, power-unrelated). Out of scope here;
    noted so Phase 0b's "one fact, one place" direction is understood as
    chipping at a codebase-wide habit, not completing it.
- **Corpus presence of the unchecked types** (per-type, corrected from rev 1's
  "6 types in 8 snapshots"): `centrifuge` in 5 snapshots, `biochamber` 2,
  `recycler` 2, `foundry` 1 — and `cryogenic-plant` / `electromagnetic-plant`
  in **zero**. No corpus case can currently exercise those two; Phase 0's
  corpus re-run cannot verify them without new fixtures.
- **Biochamber is not electric** (draftsman ground truth:
  `energy_source.type == "burner"`, `fuel_categories: ["nutrients"]`). It
  draws zero grid power, yet sits in the canonical list — and the codebase has
  **no energy-source concept at all** (zero hits for
  energy_source/needs_power/is_electric). A naive "sync the validator to the
  layout list" would therefore be actively wrong for biochamber, and
  `place_poles` today reserves pole lines for pure-biochamber rows that need
  none.
- **Reproducibility caveats**: (a) the 6 `census_*_science_pack` snapshots came
  from an uncommitted scratchpad dump script — no committed test regenerates
  them (Phase 0d fixes this); (b) a byte-faithful replay of `place_poles`'
  greedy probe failed to reproduce real pole positions in 34/45 snapshots
  (cause unknown; no replay artifact is committed). The census therefore
  measures slack from ground-truth positions, and Phase 2 is designed to never
  need the replay.

## Design

- **Phase 0 — energy-source model + list unification.** The judgment phase,
  not a mechanical sync:
  - **0a.** Introduce the codebase's first machine energy-source fact:
    `needs_electricity(entity) -> bool` in `common.rs` beside `machine_dims`
    (biochamber `false`; the other 11 `true`; ground truth from game data,
    pinned by a test).
  - **0b.** Unify the lists: delete the `lane_planner` duplicate in favor of
    `common::MACHINE_ENTITY_NAMES`; `validate/power.rs` checks canonical ∩
    `needs_electricity`; `validate/fluids.rs` checks canonical ∩ has-fluid-ports
    (adding foundry/cryogenic-plant/electromagnetic-plant). Extend the existing
    drift test (common.rs:431) so any newly added machine type fails the build
    until classified for both energy and fluid ports.
  - **0c.** Decide the pole-placement trigger for burner machines: rows are
    single-recipe, so pure-biochamber rows plausibly need no pole line at all —
    dropping biochamber from the pole trigger (layout.rs:490) is free slack.
    This is layout-moving: own STRESSGOLD gate, movement scoped to
    biochamber-row cases (self-loop fixtures, logistic-science).
  - **0d.** Commit the science-pack snapshot-dump path (an `#[ignore]`d test)
    so all 45 census snapshots regenerate from committed commands; add either
    a small fixture exercising `cryogenic-plant`/`electromagnetic-plant` or an
    explicit "unverified post-Phase-0" caveat in CLAUDE.md's validator notes.
  - Rev 1 called this phase "a plain validator bug fix [that] could land ahead
    of acceptance" — retracted; 0a–0c carry design decisions. 0d is mechanical.
- **Phase 1 — principled fluid-row reservation.** Computable at stamp time:
  `place_poles`' candidate rows are a static function of row geometry
  (layout.rs:918 — `top_y - 1` and `top_y + mh`), which templates already
  know. Mandated shape: extract a shared `pole_candidate_ys(top_y, mh)` (or
  equivalent) called by **both** `templates.rs` and `place_poles`, so the
  reservation rule and the placer can never drift — the geometry dual of the
  Phase 0 name-list fix. Today `templates.rs:3190`'s gap comment ("leaving
  dx=2 free for place_poles") is an independent hand-guess of that formula.
  Then: reserve gap tiles for **all** fluid RowKinds when the row's free-tile
  budget within the shared candidate rows falls below what coverage needs,
  sized by machine footprint. Scope: fluid rows only; solid-row behavior must
  not change.
- **Phase 2 — slack guardrail via live instrumentation.** Emit a per-pole
  trace event from `place_poles`' existing probe loop (layout.rs:938) at
  decision time, recording the free-candidate count it just observed — no
  replay, no reconstruction. Scoreboard line per case: zero-slack pole count,
  median slack, and total pole count. Live emission makes rev 1's
  replay-divergence kill criterion moot by construction.
- **Phase 3 — substation rows / dedicated power columns (deferred,
  trigger-gated).** Triggers: **(a)** any solid-row zero-slack pole appearing
  in the Phase 2 scoreboard (deterministic engine; currently zero in n=1881,
  so any appearance is signal), or **(b)** any case's total pole count growing
  >20% cumulatively from the 2026-07-19 census baseline without the causing
  diff explaining it — the pole-count-explosion failure mode that trigger (a)
  cannot see (more poles reads as *more* slack). Connectivity degradation is
  already watched by `check_pole_network_connectivity` in every corpus run.
  Until a trigger fires, no design work. (Census geometry for whenever it
  does: row height mode 7 (207/401) with 8 nearly as common (126/401);
  inter-row gaps 0 or 2 in ~84% of cases, tail to 25; an 18×18 substation
  supply spans several row cycles.)

## Kill criteria

- If Phase 0's corpus re-check (with the widened lists) reveals ≥1 real
  coverage or fluid-port failure, everything else pauses until it's fixed; if
  the post-fix re-census moves any Motivation fraction by more than 10
  **percentage points** or any count by more than 10% relative, Phases 1–3
  must be re-derived from the new numbers before proceeding — not patched
  incrementally.
- If Phase 0c moves goldens on any case that contains no biochamber rows, the
  change is leaking — stop and investigate before re-blessing.
- If Phase 1's rule cannot hold current row pitch (any corpus fluid row needs
  extra row height to fit the reservation), **stop** — the footprint-vs-power
  trade-off goes to the user, not into code.
- If Phase 1 moves goldens on any non-fluid row, the rule is leaking beyond
  its scope — stop and re-scope before re-blessing anything.
- Phase 2 fixed point: on unchanged post-Phase-1 main, the live-instrumented
  zero-slack counts must match a fresh ground-truth census within ±1 pole per
  case. Larger divergence means the instrumentation measures something other
  than reality — stop. (Replaces rev 1's replay-timebox criterion, mooted by
  live emission, and rev 1's ≤5% runtime budget, which review showed was
  non-discriminating — its cited precedent never actually measured runtime.)

## Verification plan

Per CLAUDE.md layout-change protocol. Phase 0: the extended drift test must
fail when a machine type is deliberately left unclassified; corpus re-run with
warning-population diff before/after the list widening; 0c gets its own
STRESSGOLD gate with movement scoped to biochamber-row cases. Phase 1:
STRESSGOLD before/after with every moved line explained (fluid rows only);
tile-level verification on the corrected worst-case list above (the
zero-alternative ranking, not median slack); browser eyeball of one
refinery-heavy layout (user). Phase 2: the fixed-point criterion above,
reproducing the census as a sanity anchor.

## Phasing

As numbered above; each phase lands separately. Phase 0 carries the design
judgment (energy-source model, two validator syncs, the biochamber pole
decision); Phase 1 is well-scoped once the shared-geometry extraction is done;
Phase 2 is a small contained diff given live instrumentation; Phase 3 is
genuinely and cheaply deferred.

## Decision log

- *2026-07-19 — rev 1 drafted from the power-placement census (this session's
  `power-census` agent; raw data now committed at
  `scripts/pole-census-2026-07-19.json`). Sequencing note: lands after
  `beltspan-lastinrow` merges, since Phase 2's baselines assume the
  post-extension corpus (it did — `0d7132c`).*
- *2026-07-19 — **adversarial review (2 reviewers, parallel): REVISE ×2; draft
  rewritten as revision 2.** Evidence review: every core statistic reproduced
  exactly from the committed JSON (nothing fabricated), but (a) the
  "worst cases" list was ranked by median slack while the verification plan
  consumes it as a zero-alternative-count ranking — 3 of 5 named cases were
  mild and 6 worse ones unnamed, including `tier4_advanced_circuit_from_ore_am2`
  and `tier5_processing_unit_from_ore_am3` (corrected; Phase 1 verification
  retargeted); (b) the 6 science-pack snapshots are unreproducible from
  committed code (Phase 0d added); (c) `cryogenic-plant`/`electromagnetic-plant`
  appear in zero corpus snapshots, so Phase 0 can never verify them without
  fixtures (caveat + fixture option added); (d) the within/beyond-span
  classification is census-coined, not engine-derived (caveat added). Design
  review: (e) four hand-synced machine lists, not two — including a sibling
  fluid-validation blind spot (fluids.rs:21) with real unchecked fluid ports on
  foundry/cryogenic-plant/electromagnetic-plant (folded into Phase 0b); (f)
  biochamber is burner-fueled (draftsman-verified) — a naive list sync would be
  actively wrong; energy-source model added as Phase 0a, the codebase's first;
  (g) pure-biochamber rows plausibly need no poles at all — pole-trigger
  decision added as Phase 0c; (h) Phase 1 IS computable at stamp time
  (candidate rows are static geometry, layout.rs:918) — shared
  `pole_candidate_ys` extraction mandated to prevent geometry drift; (i) Phase
  2 switched to live decision-time instrumentation, mooting the replay risk;
  the ≤5% runtime criterion replaced with a discriminating fixed-point
  criterion after review showed the cited precedent never measured runtime at
  all; (j) Phase 3 trigger extended with a pole-count-explosion watch, the
  failure mode the zero-slack trigger structurally cannot see. Phasing honesty
  inverted: Phase 0, not Phase 1, is where the judgment lives. Pending user
  acceptance.*
- *2026-07-19 — **revision 2 accepted by the user** ("i'm happy with this
  rfp"); user intends to set it as the session goal. Work proceeds phase by
  phase under the implementer + adversarial-reviewer team flow established on
  `beltspan-lastinrow`.*
