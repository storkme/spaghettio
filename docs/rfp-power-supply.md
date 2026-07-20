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
  - **0e (added 2026-07-19, user-directed scope change) — fluid-port
    correctness before pole design.** The widened Phase 0 validation exposed
    two real defects that must be fixed before Phase 1 designs reservations
    around fluid-row geometry: **(i)** fluid row templates route pipes to a
    fixed north face, but cryogenic-plant's inputs are south and
    electromagnetic-plant's are west/east — the templates must honor
    per-machine port faces (bounded parameterization, see the kill criterion
    below). Fixtures come from the Phase 0 repro params (superconductor@AM3,
    fusion-power-cell@AM3) and land with the fix, retroactively closing 0d's
    fixture gap. **(ii)** The fulgora holmium-chain chemical-plant output gap
    (melted water unpiped at the chemical-plant, hidden because the fulgora
    test validates sushi belts only) — widen that test to full validation and
    fix the routing. Both are layout-moving: separate units, standard gates,
    one in flight at a time.
  - **0f (added 2026-07-19 on the KC1 re-derivation trip, user-accepted) —
    inserter power coverage.** Adversarially confirmed: 40–52% of electric
    inserters corpus-wide sit at nearest-pole distance exactly 4 (one tile
    beyond the ±3 supply area) because `place_poles` targets machine centers
    from the north band; inserters are not coverage subjects anywhere in
    validate/. One unit, two halves landed together: **(i)** electric
    inserters become power-coverage subjects; **(ii)** `place_poles` covers
    inserter rows, not machine centers only (the uniform distance-4 signature
    suggests south-band or two-band placement). Landed as ONE unit
    deliberately — splitting would leave the corpus either falsely green or
    wearing hundreds of honest-but-transitional red warnings across commits;
    the unit's report shows the intermediate red, and the landing is green.
    Expect the largest golden movement of this RFP (every layout gains or
    moves poles) — full team flow plus user eyeball. **After 0f lands:
    re-census, then re-derive Phase 1's reservation formula inputs and
    Phase 2's fixed-point baseline from the corrected corpus; Phase 3
    trigger (b)'s pole-count baseline re-anchors to the post-0f census**
    (otherwise the fix itself would trip the explosion trigger it
    legitimately feeds). Sequencing: 0e → 0f → re-census → 1 → 2. Computable at stamp time:
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
- If Phase 0e's port-face fix cannot be a bounded parameterization of the
  existing fluid templates — i.e. it starts requiring the wholesale fluid-row
  redesign of [#68](https://github.com/storkme/spaghettio/issues/68) /
  `docs/rfp-fluid-dual-input-row.md` — **stop**; that is a separate RFP and
  the user decides sequencing.
- If Phase 0f cannot cover inserters within the current row pitch — i.e.
  pole positions require new rows/columns beyond today's free tiles — that is
  the Phase 3 substation trigger arriving early: **stop** and take the
  substation conversation to the user rather than forcing coverage into
  geometry that doesn't want it.
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
- *2026-07-19 — **Phase 0a/0b/0d LANDED** (`d955434`/`35d9d5a`/`ed7cf6a`
  post-rebase; implementer + adversarial-review fork, verdict APPROVE). Kill
  criterion NOT tripped: corpus re-check clean, both-direction position-keyed
  warning diff 0 added / 0 removed, STRESSGOLD byte-identical 8/8, drift test
  demonstrated failing-then-passing. Review catches recorded: the fluids guard
  swap correctly removed a false positive the old allowlist emitted — the
  `ice-melting` recipe (ice→water, zero fluid ingredients) refutes the deleted
  comment's "every chemical-plant recipe requires at least one fluid" premise;
  and the census's USP fidelity figure (6615 entities) predates the
  `0d7132c` belt extension — post-extension corpus value is 6619 (dims
  208×281 unchanged), Phase 0 innocent. Two real defects surfaced by the
  widened validation: the cryo/electromag port-face routing gap and the
  fulgora chemical-plant output gap.*
- *2026-07-19 — **user decision: fold both defects into this RFP as Phase
  0e**, ahead of Phase 1 ("i don't think there's much point in us finding a
  solution to power poles only to have it not work"). Rationale sharpened in
  design terms: Phase 1 would otherwise bake broken port geometry into the
  reservation rule's assumptions. 0d's "unverified post-Phase-0" CLAUDE.md
  caveat is superseded (fixtures arrive with 0e); no external tracking issues
  filed while the work sits inside the active session goal. New kill
  criterion added: if the port-face fix escalates into the #68 fluid-row
  redesign, stop and return sequencing to the user.*
- *2026-07-19 — **Phase 0c decided: KEEP biochamber in the pole trigger; no
  layout change** (decision unit; adversarial review CONFIRM-KEEP). The RFP's
  free-slack premise is disproven: a biochamber row's only grid consumers are
  its electric inserters, and the row's pole line is what powers them.
  Drop-experiment evidence (implementer, synthetically re-verified by
  reviewer): pure rows lose all poles (pentapod 6/6 inserters stranded,
  bacteria 3/3 — visible "No power poles" warning); mixed layouts strand
  biochamber-row inserters with the validator **coverage-silent and
  connectivity-fickle** (0 coverage warnings both ways; a connectivity
  warning fires only when the removed poles happen to be wire bridges).
  Defensive comment added at the layout.rs pole-subject loop.*
- *2026-07-19 — **KC1 re-derivation clause TRIPPED (in spirit) by a systemic
  finding out of 0c's investigation, adversarially confirmed**: 40–52% of
  electric inserters across normal all-electric corpus layouts sit outside
  pole coverage under the engine's own ±3 Chebyshev model (metric
  draftsman-derived: supply_area_distance 3.5 on a 1×1 pole ⇒ ±3 exact for
  1×1 entities; reviewer recounts EC@10 60/140, gear@5 10/20, mixed chain
  67/168; machine centers 0% uncovered everywhere — the calibration anchor).
  Signature: every uncovered inserter is at nearest-pole distance exactly 4 —
  `place_poles` targets machine centers (`cxs`, layout.rs:936) from the north
  band (top_y−1, layout.rs:926), putting 3×3-machine south inserter rows at
  Δy=4. Latent because inserters are not power-coverage subjects anywhere in
  validate/ and the inserter-sizing RFP's in-game import anchors were never
  run. Consequences: "0 power warnings", the census slack distributions, and
  Phase 1's "what coverage needs" formula all understate true pole demand.
  Sequencing decision passed to the user.*
- *2026-07-19 — **re-scope accepted by the user** ("agree with the re-scope,
  please continue"): Phase 0f (inserter power coverage, one combined
  validator+placement unit) added; ordering 0e → 0f → re-census → Phase 1
  (re-derived) → Phase 2. Phase 3 trigger (b)'s pole-count baseline will
  re-anchor to the post-0f census. Kill criterion added: if inserter
  coverage can't fit current row pitch, the substation conversation starts
  early with the user.*
- *2026-07-19 — **0e scope refinement + re-sequencing (user call)**. Implementer
  diagnosis before code: 0e's two items share one root cause — fluid-row
  templates hardcode port positions (input assumed north, output assumed
  solid/south) instead of reading per-machine geometry. Fulgora's bug is the
  classifier: `row_kind` ignores fluid OUTPUTS, so solid-in/fluid-out recipes
  (ice-melting, foundry molten-metal) land in solid-output templates that
  never pipe the fluid. The clean unified fix for the port-face half is a
  per-machine port-geometry parameterization across all four fluid templates
  + validator orientation-awareness — bounded (no #68 pitch redesign) but
  sizable, and for electromagnetic-plant solid-output-only (fluid-output
  recipes like electrolyte stay unsupported until #68; they fail LOUD post
  Phase 0, which is what makes deferral safe). User decision: **0e-ii
  (fulgora, green-lit, in flight) → 0f → 0e-i (port-face parameterization,
  both fixtures) → re-census → Phase 1 → Phase 2** — the inserter-coverage
  defect outranks the port-face refactor, and Phase 1's premise no longer
  depends on (i) since unsupported rows can't be generated silently anymore.
  Foundry molten-metal support travels with 0e-i.*
- *2026-07-19 — **Phase 0e-ii (fulgora) LANDED** (`cbf5017`; adversarial
  review APPROVE, retargeted mid-review after a branch reconciliation — a
  freeze-during-review rule now stands). Review evidence: cherry-pick
  equivalence proven; STRESSGOLD 8/8 byte-identical; gate enumeration shows
  ice-melting is today's only firing recipe; the widened fulgora test FAILS
  against base (real regression guard); the water arc also cleared 2
  incidental warnings from the phantom output belt (109→107). Review
  findings folded forward: **biolubricant** (biochamber, jelly→lubricant) is
  a third solid-in/fluid-out member, added to 0e-i's scope (likely a
  one-line gate widening — biochamber's ports mirror chemical-plant's);
  0e-i should consider promoting fluid-output into `row_kind` itself once
  2–3 templates carry per-template fluid arms (design-seam note,
  placer.rs:502–545); the fluid-validators-only test widening is ACCEPTED
  (46 belt-loop + 53 UG errors are the documented sushi design, identical at
  base and branch) with a "full-validate-minus-sushi-region" test noted as
  future work. Pre-existing fulgora sushi defects (3 illegal entity
  overlaps at (29,14)/(29,16)/(29,17) + the AM3 (8,45) single-exit-bus
  cluster) filed as a tracking issue — out of this RFP's scope.*
- *2026-07-19 — **0f kill criterion CONFIRMED by narrow-scope adversarial
  verification; user decision: LAND with honest reds.** Verifier checked
  every uncovered inserter (not a sample) against final post-routing
  occupancy: zero free tiles in each one's entire 7×7 — a substation's 2×2
  wouldn't fit post-hoc either; only template-level reserved power space
  reaches these. The packed 7-row cycle (3 belts + inserter row + 3×3
  machines, zero slack) puts inserter rows at distance exactly 4 from the
  nearest possible pole row on both sides. Implementer's full-corpus tables
  then widened the trip from two cases to THREE: processing-unit@2 (24 per
  implementer / 20 per verifier — reconciliation is a merge-review item),
  utility-science@1 (16), and uranium-235@0.1 kovarex self-loop recirc rows
  (16, new find, same 0/49-free signature). Everything else lands 100%
  covered, machines 0-uncovered corpus-wide, biochamber rows clear, pole
  network single-component everywhere. Pole cost ≈2× corpus-wide (EC@10
  30→59, PU 163→331, util-sci 134→278; committed-corpus-anchor caveat noted
  for fluid rows) — this is the Phase 3 trigger-(b) re-anchor input.
  **Phase 3 is hereby ACTIVATED as scheduled design work** (no longer
  dormant): it inherits three live fixtures exhibiting exactly the failure
  it must solve (deep decomposed stacks + self-loop recirc); scheduling
  relative to Phase 1/2 to be decided at 0f wrap. Rationale for landing
  (verifier + lead concur): holding buys nothing — substation work re-moves
  the same goldens regardless, and the corpus-wide coverage win + honest
  validator bank now.*
- *2026-07-19 — **Phase 0f LANDED** (`3ae17e2`/`f0f2822`/`4a64f3c`/`e7c9a79`;
  review arc REVISE → reorder fix → APPROVE; user eyeballed and approved).
  Electric inserters are now power-coverage subjects (via the single
  `needs_electricity` fact); `place_poles` runs two outward candidate bands
  seeded from `common::pole_candidate_ys` plus a greedy set-cover mop-up; and
  the pipeline finally honors its own stated invariant — **poles are placed
  LAST, after `route_bus_ghost`, and are never router obstacles**. The
  review's logistic-science regression exposed that poles-before-routing had
  been quietly violating the RFP's "poles live off leftover tiles" premise;
  after the reorder, routing is pole-independent: non-pole entities are
  byte-identical to base across the entire corpus except pentapod, whose old
  2-PTG water bridge was itself a pole artifact (base had a pole dead-center
  in the water run). Honest-red pins, all hardness-scanned 0-free and
  suite-asserted: EC@20 ×14, EC@60-red ×60 (ceiling=60; exact-shape stress
  assertion is a follow-up), PU-am3 ×20, PU-am2-baseline ×43, kovarex ×16.
  Known-hard non-gating: USP ×16, chemical@10 ×23 (gauntlet). Pole cost at
  committed configs: corpus 1770 → 3464 (+96%); gear@5 5→11, EC@20 62→117,
  EC@35 119→232, EC@60-red 179→357, AC-am2 82→162, PU-am3 181→343. Phase 3
  trigger-(b) baseline re-anchors to the post-0f re-census (pending).
  Follow-ups: delete the vestigial `pole_entities` router param (dead code);
  per-pack gauntlet assertions gap; exact-shape stress assertion.*
- *2026-07-19 — **post-0f re-census complete** (agent-run on `debb398`; raw
  data `scripts/pole-census-2026-07-19-post0f.json`, analysis script now
  committed as `scripts/pole_census.py` — closing the pre-0f census's
  script-reproducibility gap). Findings: **(A)** power warning footprint
  matches the pinned reds exactly, zero deviation. **(B) Phase 3 trigger
  (a)'s condition is now TRUE in the corpus**: 18 solid-row zero-slack poles
  (pre-0f: 0/1881), all on advanced-circuit south-band output-inserter rows
  across 13 cases — the two-band rework spent the slack the original census
  measured (solid median 8→4, fluid 4→2; fluid zero-slack 38%→30.4%
  proportionally but 108 absolute). Phase 3 is already activated, so no
  state change — recorded as reinforcing evidence and as the first entry of
  the trigger-(a) watchlist. **(C) trigger-(b) baseline re-anchored**:
  corpus poles 2212→4226 (+91%, 45-snapshot scope; the landed cost table's
  3464 is the 39-committed-config scope — both scopes now named). Missing
  fixtures filled: kovarex 4→5 (+25%), PU-am2-baseline 99→190 (+92%). The
  older spot figures in the "trip confirmed" entry (EC@10 30→59, PU
  163→331) were pre-reconciliation config-drift numbers superseded by the
  landed table — noted here so the log's two sets don't read as an open
  contradiction. **(D)** in-span fraction 82%→~79-80%, inside the 10pp
  re-derivation band. **(E) Phase 1 premise holds**: fluid rows remain
  categorically tighter (30.4% vs 0.5% zero-slack); worst fluid rows are
  all basic-oil-processing refinery rows (ranked list in the JSON). One
  re-read flagged: Phase 1's "moves goldens on any non-fluid row → stop"
  criterion now operates in a corpus where solid rows are already imperfect
  — the advanced-circuit tightness is Phase 3's problem, not Phase 1's, and
  Phase 1's scope stays fluid-only.*
- *2026-07-20 — **Phase 0e-i LANDED** (`e58e1a6`..`20f475b`, 6 commits
  rebased onto the re-census docs; adversarial review APPROVE, zero blocking
  findings). Shared `crate::fluid_ports` module: validator AND templates now
  consume one orientation-aware port-table source (mirror + direction), with
  a committed draftsman provenance script
  (`scripts/verify_fluid_ports_transforms.py`); the reviewer re-derived
  every table and transform from raw fluid_boxes data — exact matches, and
  the foundry/cryo x-symmetry makes mirror ≡ 180-rotation unambiguous.
  Landed: cryo/foundry mirror rows, emag East-rotation (solid-output only;
  electrolyte verified STILL failing loud), foundry molten-metal DualInput
  fluid arm, biolubricant via a principled south-face gate. Four new
  fixtures give emag/cryo/foundry-molten/biolubricant their first corpus
  presence — 0d's fixture gap now fully closed. **Design-seam ruling**
  (review adjudicated the implementer's dissent): per-template fluid-output
  arms + shared `emit_fluid_output_row` helper, NOT row_kind promotion —
  output_is_fluid has one central derivation feeding all arms, the gate
  cannot drift from the tables by construction, and emission geometry is
  irreducibly per-template. Superconductor's 6 pinned warnings adjudicated
  pre-existing (the FluidInput template's standing 3-solids limitation,
  first exposed by the new fixture — a follow-up candidate, not 0e debt).
  Corpus byte-identical (goldens 9/9, purely-additive e2e diff). Phase 0e is
  COMPLETE; Phase 1 dispatches next with the re-census inputs.*
- *2026-07-20 — **Phase 1 LANDED** (`c91e98f`; light-scoped adversarial
  review APPROVE + ACCEPT-SCOPE). The `msz==5 && oil-refinery` special case
  is replaced by footprint-derived `fluid_row_pole_gap_dx` with a live
  `debug_assert` linking the gap to `common::pole_candidate_ys` (drift fails
  the suite). **Corpus outcome: byte-identical — and that is the correct
  Phase 1.** The RFP's "expect moved fluid-row lines" prediction is
  FALSIFIED and recorded: the refinery (the corpus's only 5×5 fluid-only
  row) already carried a hand-rolled gap, and 3×3 fluid rows are covered
  from an adjacent band tile (verified at two rates; reviewer notes the
  covering pole sits either in the row above or beyond the row's end —
  both within ±3). Reviewer tile evidence: 5×5 strips read P-U-O-U-P (full
  except the derived gap), covering pole at Chebyshev exactly 3.0 — the
  reservation is geometrically forced, not conventional. Speculative 3×3
  reservation was declined (lead scope call, reviewer-affirmed on evidence):
  works-but-fragile fluid poles (108 zero-slack) are Phase 2's
  measure-then-react territory, not Phase 1 pre-emption. Forward-safe: a
  future 5×5 fluid-only machine gets its gap automatically; the msz==4 arm
  is unreachable today and fails LOUD (coverage warning) if ever reached.
  Phase 2 dispatches next.*
- *2026-07-20 — **Phase 2 LANDED** (`43c690b` instrumentation + `db2bcb9`
  separable golden re-bless; light adversarial review APPROVE). Live
  per-pole `PoleSlack` trace emission + three scoreboard lines per case
  (total poles, zero-slack, median slack) — every future densification
  change now pays its power cost visibly in the committed golden diff.
  **Deviation ruled ACCEPT and recorded**: the RFP's Phase 2 spec was
  internally inconsistent — "decision-time" emission cannot ±1-match a
  census that computes slack post-hoc over the complete pole set (a
  per-decision measure is blind to later mop-up/repair poles and would
  systematically overcount slack). Resolved toward the falsifiable
  requirement: post-placement-live over final positions, which is also the
  semantically correct guardrail (densification must pay for FINAL-state
  fragility). Fixed point verified twice (implementer 12 cases, reviewer 4
  independent re-derivations): zero-slack matches EXACTLY everywhere; the
  one total-count off-by-one (partitioned_5s 92 vs 93) demonstrated to be
  the two-strategy fixture's pooled-vs-partitioned leg, not instrument
  error. Layouts byte-identical (+3 golden lines/file only); suite +~4%
  runtime, honest. Crash-recovery note: the implementer died 3× on
  provider 500s mid-phase; a lead protective WIP commit preserved the
  work, verified fully carried into the clean recommit (`git diff` empty
  modulo documented upgrades). Merge-time fixes: `pole_census.py`'s
  hardcoded snapshot path made env-overridable (reviewer finding 1).
  Follow-up noted, not blocking: a solid/fluid split of the zero-slack
  scoreboard line would make Phase 3 trigger (a) literally readable
  (aggregate currently serves as a conservative tripwire). **Phases 0, 1,
  and 2 of this RFP are all LANDED.** Remaining: Phase 3 (design accepted
  and committed, docs/rfp-power-reservation.md) and the trim rider (in
  flight).*
- *2026-07-20 — **Phase 3 COMPLETE** (full arc in
  `docs/rfp-power-reservation.md`: 3a-i, 3a-ii, 3b, 3c). Phase 3 delivered
  **reactive band-widening** that clears all five hard cases — EC@20, EC@60-red,
  PU-am3, PU-am2-baseline via **medium poles** (the freed +2 band lands the
  covering pole at distance exactly 3), kovarex via the **substation fallback**
  (its 5-row-deep top-edge recirc is beyond medium ±3). The RFP's
  substation-necessity premise was **falsified for 4/5** (medium suffices
  post-widen — the dual-input row has 2 belt rows, not the assumed 3) but
  **vindicated for kovarex** and, as the post-3b re-census surfaced, for the
  non-gating USP (a second deep-geometry substation case); the substation
  machinery is thus **exercised in two real corpus cases, not dead code**. The
  **corpus is at ZERO pinned honest-red power warnings** (the arc opened with 153
  gating + 16 non-gating). Phase 3c re-anchored **trigger (b)** to the post-3b
  49-case baseline (4251 poles; matched-45 growth +9 / +0.21%, every delta a
  Phase-3 case, 40/45 cases byte-identical) and confirmed **trigger (a)**
  unchanged (18 solid zero-slack, already-activated). Two review followups
  landed: a `ReactivePassNotConverged` convergence signal (so a new starved case
  can't ship uncovered without an alarm) and a zero-margin comment at the widen
  constant. Re-census data + reproducible analysis:
  `scripts/pole-census-2026-07-20-post3b.json` +
  `scripts/pole_census_analysis.py`. Validator-verified only; the in-game import
  anchor remains a user step.*
- *2026-07-20 — **THE WIRES ARC — recorded out-of-band (arc review's biggest
  record gap).** This arc happened between Phase 3a-ii and 3b and never had its
  own RFP phase, so it lived only in commit messages until now. An **in-game
  paste by the user** exposed the largest latent defect of the entire power
  arc: **every blueprint this generator has ever exported encoded NO pole
  copper wires.** The export emitted poles as bare entities with no `wires`
  array, so a pasted factory's poles were **disconnected islands and the whole
  layout was power-dead on arrival** — no matter how perfectly `place_poles` +
  `repair_pole_connectivity` had arranged them geometrically. Fixed across four
  commits: `a7d9a48` (blueprint-level `wires` array — `[a, 5, b, 5]`
  pole-to-pole copper, connector id **5** = draftsman
  `WireConnectorID.POLE_COPPER`, the Factorio 2.0 format; the old 1.x
  per-entity `neighbours` array is not read by 2.0), `cdf71e8` (export /
  round-trip tests + the corpus-wide single-connected-component invariant),
  `6e21bd4` (medium-pole footprint correction + the 2.0 wires export rule
  documented), and `57d43d1` (web power-connectivity overlay so the wire graph
  is visible in the app). **A semantic change rides along:**
  `check_pole_network_connectivity` was rewritten from a **geometric-proximity**
  test (do poles sit within reach of each other?) to an **artifact-level** test
  (is the graph in the *emitted* `wires` array one connected component?), now
  reading the same `crate::power_wires` module the export and web overlay
  consume — draftsman-verified, round-trip-locked. **This retroactively
  reframes every earlier "single-component" / "pole network connected" claim in
  this arc's log as GEOMETRIC-ONLY** — true of the pole positions, silent about
  the then-absent wires. The discovery is the **single strongest argument this
  project has for the in-game import anchor**: a defect that made 100% of
  exports non-functional survived the whole 23-check validator suite, the
  census, and every "0 power warnings" reading, because nothing short of a
  human paste could see it.*
