# RFC-062: Multi-target output support

**Status**: Partial — engine (solver + layout, Phases 0–2) and Phase 3
tooling (wasm/URL/harness) land and hold; the final gate's kill criteria
3 and 4 both FAIL as measured on the canonical fixture (KC4: real, ties
on machines, loses on area/entities; KC3: real, but isolated to a
newly-found sim-harness instrument gap, not proven to be a layout
defect). See the close-out decision-log entry (2026-08-01) for the full
adjudication and enumerated deferred items.
**Tracking**: (opens on PR)
**Registry**: [`rfcs.md`](rfcs.md)

## Summary

Let the solver and layout engine accept **N ≥ 1 user-specified targets** in
one factory instead of exactly one. Canonical case: `electronic-circuit@10/s`
**and** `advanced-circuit@3/s` from ore, sharing raw-ore mining, smelting,
and — the case that actually matters — copper-cable production, in a single
blueprint with two export belts. Today `target_item`/`target_rate` are
scalar from the wasm boundary down through every layer of the solver and
layout engine, so asking for two targets today means running the whole
pipeline twice and gluing the results together by hand. This RFC's Phase-0
probe (Motivation) shows the cheap way to glue them is silently wrong and
the safe way is ~10% overbuilt — both are worse than a real combined solve,
which is the case for building this.

## Motivation

There is no way to ask spaghettio for two simultaneous targets today. The
complexity ladder in [`docs/status.md`](status.md) (§"Recipe complexity
ladder") is single-target throughout, and no existing issue, RFC, or doc
addresses simultaneous multi-target solving or layout — this is new ground,
not a gap in something already attempted.

The natural workaround — solve each target independently and place the two
results side by side — was probed directly on the canonical case
(`electronic-circuit@10/s` + `advanced-circuit@3/s`, from ore, AM2,
2026-07-31, session-run; the probe script is gitignored/local, so the table
below is the durable record):

| Approach | Copper-cable machines (LP, pre-integer-snap) | Total machines (LP, pre-integer-snap) | Correct? |
|---|---:|---:|---|
| **Naive concatenation** — solve `EC@10/s` and `AC@3/s` independently, place side by side | 20.0 | **137.9** | Correct, but shared upstream (ore → plates → cable) is sized and built **twice** — ~10% overhead |
| **Hand-sum shortcut** — notice AC needs `EC` as an ingredient, solve one combined `EC@16/s` (10 target + AC's 6/s ingredient demand) and reuse its plan for both | **16.0** | — (plan is wrong before totals matter) | **WRONG** — undercounts copper-cable by 4 machines. `EC@16/s`'s own recipe prices copper-cable for making EC, but AC's recipe *also* consumes copper-cable directly (2 cable per AC craft) — a second coupling edge the hand-sum solve never sees, because it only ever asked for `EC@16/s`, not for AC at all |
| **True combined requirement** — what a real multi-seed solve must reproduce | 20.0 | ~124.3 (estimated; not yet measured by an actual multi-seed LP run) | Correct **and** deduped |

The two failure modes bracket the design problem exactly. The hand-sum
shortcut is a **correctness bug**: it looks like a free optimization
(reuse one solve for the shared item) but silently drops a real demand
edge — nothing downstream would catch a factory built to this plan short
of it starving in-game, which is exactly the failure class
[`docs/validator-reporting.md`](validator-reporting.md) exists to prevent
recurring. Naive concatenation is **correct but wasteful** — it never
misses an edge because each solve independently walks its own full recipe
tree, but it can't see that the two trees overlap, so it builds the
overlap twice. Only a real multi-item solve — one LP with both targets as
simultaneous demand rows — discovers every coupling edge (including the
ones a human eyeballing the recipe graph would miss) while deduping the
shared upstream. That is the core argument for doing this in the LP rather
than at the orchestration layer.

## Design

### Solver (`crates/core/src/netflow.rs`, moderate effort — mechanical, but wide)

The net-flow LP is already **item-indexed internally**: one flow-conservation
row per item, with RHS `target_rate if i == target_idx else 0.0`
(`netflow.rs:799`, confirmed against origin/main 2026-07-31). Generalizing to
N targets is conceptually "make three single-item seeds into N-item ones,"
but the call graph fanning out from `target_item: &str, target_rate: f64` is
wide: **8 public wrappers in `solver.rs`** (`solve`, `solve_with_palette`,
`solve_with_exclusions`, `solve_with_palette_and_exclusions`,
`solve_compat_with_palette_and_exclusions`,
`solve_free_with_palette_and_exclusions`,
`solve_with_palette_exclusions_and_quality`,
`solve_with_palette_exclusions_quality_and_modules`) plus **3 layered
entry points in `netflow.rs`** (`solve_netflow`, `solve_netflow_with_options`,
the private `solve_attempt`) all thread the same scalar pair as parameters.
Every one of those 11 is a thin forwarding wrapper — the single choke point
that actually builds the LP is `solve_attempt` (`netflow.rs:430`), reached
through `solve_netflow_with_options`'s cycle-exclusion / oil-path-physical
retry loop (`netflow.rs:344-427`), which is itself already
target-count-agnostic (it inspects `r.machines` / `r.surplus_outputs`
post-solve, never `target_idx` directly).

Plan: add one new multi-target entry point that takes
`targets: &[(String, f64)]` (or a small `struct Target { item: String, rate:
f64 }`) and thread it through `solve_netflow_with_options` →
`solve_attempt`, replacing the internal single `target_idx`/`target_rate`
locals with a `Vec<(usize, f64)>` of interned target indices and rates.
The 11 existing scalar wrappers become one-element-slice callers of the new
entry point rather than a parallel implementation — this is what makes kill
criterion 5 (N=1 bit-identical) a **construction guarantee**, not a tested
coincidence: there is only one code path, and the N=1 case is a literal
special case of it, not a maintained duplicate.

Three concrete sites inside `solve_attempt` change from single-item to
N-item:

1. **Row RHS assembly** (`netflow.rs:799`) — `let rhs = if i == target_idx {
   target_rate } else { 0.0 };` becomes a lookup into the target-rate map,
   `targets.get(&i).copied().unwrap_or(0.0)`. (Found during this RFC's
   verification pass; not called out in the original research dossier, which
   named only the two seeds below — it is the same shape of change and
   belongs in the same commit.)
2. **Demand-closure seed** (`netflow.rs:527,538`) — `demanded[target_idx] =
   true` becomes a loop seeding `demanded[idx] = true` for every target
   index before the closure fixpoint runs.
3. **Output-assembly DFS seed** (`netflow.rs:1044`) — `let mut stack =
   vec![Work::Item(target_idx)];` becomes `targets.iter().map(|&(idx, _)|
   Work::Item(idx)).collect()`.

`SolverResult.external_outputs` is already `Vec<ItemFlow>` (`models.rs:123`,
docstring: *"The requested target item(s) only"* — already written for this
day) but both entry points currently hardcode a one-element vec:
`netflow.rs:1159-1164` (the real N-target construction site — becomes a
loop over `targets`) and `solver.rs:347` (the **legacy recursive tree
walk**'s own construction, `solve_tree_walk_with_palette_and_exclusions`,
`solver.rs:308-353`). The legacy walk is the recipe-*selection* compat/parity
oracle only (`solver.rs:308-317`, "do not add new callers") and is never
exposed to multi-target callers — its hardcode is the same shape but does
**not** need generalizing; it is listed here for completeness so a future
reader doesn't mistake it for a missed site.

**Correctness gap found in this RFC's research, must be closed before
Phase 2 (layout) is trustworthy**: `detect_di_couplings`
(`netflow.rs:1198-1287`) currently qualifies an item for a direct-insertion
coupling when it has exactly one active producer column, exactly one active
consumer column, no external supply (`s_of(i)`), and no surplus
(`o_of(i)`) — all correct for single-target, because today's target item is
never itself a producer/consumer *column* (its export is a property of the
row's RHS, not something `detect_di_couplings` observes). Under multi-target,
an item can be **both** one of the N targets **and** 1:1-coupled to a single
internal consumer (exactly the EC↔AC shape in the canonical case if EC's
production happened to route only through AC with nothing left over). If
`detect_di_couplings` stamps that pair as DI, the placer may fuse producer
directly into consumer with no exposed belt — and the item's export path,
which the RHS still demands, has nowhere to attach. Fix: `detect_di_couplings`
must take the target item-index set and refuse to couple any item that is a
member of it, regardless of what `s_of`/`o_of` say. This is a small,
targeted change, but it is silent-wrong if missed — exactly the shape
`docs/validator-reporting.md` warns is easy to miss because nothing about it
raises an error; it just ships a factory with no export belt for one of the
two things the user asked for.

### Layout (`crates/core/src/bus/`, moderate effort, concentrated in one seam)

Most of the pipeline is already N-ready:

- **`ghost_router.rs` Step 7** (`:3421-3516`, confirmed) already loops `for
  item in &output_items` built from `solver_result.external_outputs`
  (`:3438-3442`), with `merge_x_cursor` east-tiling each item's export belt
  east of the last (`:3466-3507`). This is live infrastructure today for
  byproduct/surplus exports — it needs no changes to carry a second *target*
  export, only for the target set feeding it to actually contain two items
  (already true once the solver change lands).
- **`final_output_items`** (`layout.rs:696-701`) is already an
  `FxHashSet<String>` built from `solver_result.external_outputs`, not a
  single string — no change.
- **Validation** is item-generic throughout `crates/core/src/validate/` —
  every check operates on entities/connections/flows keyed by item name, not
  on "the" target. Zero changes needed for the 36 existing checks, other than
  the one new invariant below.

**The hard problem, narrow and named**: a row's output belt gets exactly
**one** fate, decided once at row-build time — `is_final`
(`placer.rs:2948-2951`, `spec.outputs.iter().any(|o| !o.is_fluid &&
final_items.contains(o.item.as_str()))`) drives `output_east: bool`
(`placer.rs:704` parameter to `build_one_row`) — export **XOR** internal
tap-off, chosen from the row's own recipe output alone, with no visibility
into whether that same item is *also* consumed by another row. Meanwhile
`lane_planner.rs`'s `item_to_consumers` map (`:211-237`) registers every
consumer of an item **regardless of the producer row's export status** —
it just scans `rs.spec.inputs` for every row. Nothing today reconciles the
two claims on the same physical belt: if EC is both a target (so its
producer row is `is_final = true`, belt routes east to the boundary) *and*
an ingredient AC's row consumes (so `item_to_consumers` expects a lane
tapping the same producer row), the two mechanisms that would each build a
correct-looking belt independently have never been exercised together, and
the dossier that motivated this RFC could not determine from static reading
alone which one wins, or whether either produces a physically valid belt.

**Closest existing primitive to generalize**: the fluid-only "dual-purpose
lane." `BusLane::perimeter_exit_y: Option<i32>` (`lane_planner.rs:83`,
confirmed) already models a lane that has **both** internal consumer rows
(`consumer_rows`, non-empty) **and** an export exit — used today for
fluid surplus that needs to keep flowing to internal consumers while also
exiting to the perimeter (`lane_planner.rs:378-393` computes the exit
offset; `:505` and `:530` branch on "exit with no consumers" vs "exit with
consumers" as distinct routing shapes). This is precisely the shape a
solid target item that is *also* an internal ingredient needs. The leading
design hypothesis is that generalizing `perimeter_exit_y` (and the routing
around it) from fluid-only to solids removes the need for a new physical
row template — but this is a hypothesis, not yet confirmed, which is why
it is the subject of the Phase-0 spike below rather than assumed in the
design.

**New validator invariant** (belongs in `crates/core/src/validate/`,
mechanically new but conceptually small): a shared row's total claimed
outflow — export rate plus the sum of every internal tap's claimed rate —
must not exceed the row's actual production rate. Per
[`docs/validator-reporting.md`](validator-reporting.md), this emits **one
positioned issue per over-claimed row**, never a count folded into a
message — the existing failure mode this repo has hit nine times is a
check that goes quiet without the underlying defect being fixed, and a
shared-row overclaim is exactly the kind of thing that can look
validator-clean while starving one of the two targets (the #519/#520
lineage of "validator-clean, wrong in-game" is the model to avoid, not just
cite).

### Export / sim harness (small)

`blueprint.rs::export_with_manifest` (`:130-174`, confirmed) is already
fully item-keyed — `"targets"` in the manifest is built by mapping over
`solver.external_outputs` (`:153-157`), so an N-target `SolverResult`
produces an N-entry manifest with zero export-side changes.

The harness has one real gap: `crates/sim-harness/scenario.rs` drives
verification off a **single `TARGET` Lua global** (`:602-611`) and
`report.rs` only computes a delivered-rate verdict for the **first**
target (`:296-322`). Both need to become per-item series — one checkpoint
stream and one verdict per target, not just the first. The just-merged
per-machine timeseries work (PR #542) is adjacent machinery worth reusing
for the plumbing shape (keyed series, not a single scalar accumulator).

### Interface (small, deliberately minimal)

- `wasm-bindings/src/lib.rs::solve` (`:251-271`, confirmed) takes scalar
  `target_item: &str, target_rate: f64`. Extending the wasm surface to
  accept a target list is real but small, following whatever shape the
  core `targets: &[(String, f64)]` entry point takes.
- `web/src/state.ts` encodes a single `item`/`rate` pair in URL state
  (`:4-5,374-375` per the research dossier). A **minimal** URL extension —
  enough to drive the N=2 canonical fixture end-to-end for verification and
  manual eyeballing — is in scope. A full multi-target editing UI (add/remove
  target rows, per-target rate controls in the sidebar) is explicitly not.
- No prior art: no existing issue, RFC, or doc addresses simultaneous
  multi-target solving or layout.

### Scope

**In**: solver N-item generalization (§Solver) and the `detect_di_couplings`
guard; layout shared-row handling generalizing the dual-purpose-lane
mechanism (§Layout) plus the new outflow-conservation validator invariant;
harness per-item verification plumbing; a minimal URL extension sufficient
to load the canonical EC+AC fixture in the browser.

**Out, explicitly**:
- A full multi-target web UI (sidebar target list, per-target editing) —
  own follow-up once the engine supports it.
- Fluids as targets, beyond whatever falls out free of the item-generic
  design — fluid export/perimeter mechanics are already more developed
  than solid export (the dual-purpose lane itself is fluid-only today), but
  this RFC does not chase fluid-target-specific gaps.
- Cell-composition multi-target (RFC-048/051 lineage) — composing multiple
  target cells is a different mechanism (sub-solve composition, not a
  shared bus). The meeting point is the future cell-interface RFC; this RFC
  does not attempt to unify with it.

## Kill criteria

1. **Shared-row behavior must be fixable by generalizing the dual-purpose
   lane, not by inventing new row templates.** Phase 0 (below) hand-builds a
   `SolverResult` with `electronic-circuit` in `external_outputs` **and**
   as an active producer feeding an internal `advanced-circuit` consumer
   row, and runs `build_bus_layout` on it directly (solver untouched). If
   the observed failure at the shared EC row can be characterized and fixed
   by extending `perimeter_exit_y`/dual-purpose-lane routing to solids, the
   design proceeds. If fixing it demands a genuinely new physical row
   template (not a routing/lane extension), stop and re-scope the layout
   half of this RFC before writing any solver code — building the solver
   side first would be exactly the "cheap phases prove nothing" trap
   RFC-057 and RFC-058 both paid for.
2. **Multi-seed solve must not reproduce the hand-sum shortcut's error.**
   On the canonical probe case (EC@10/s + AC@3/s from ore, AM2), for every
   item consumed by more than one target's recipe tree — copper-cable
   being the measured instance — the multi-seed LP's computed machine/flow
   total for that item must exactly match what **naive concatenation**
   computes for it (20.0 copper-cable machines, not the hand-sum shortcut's
   16.0). This is a correctness bar, not an efficiency one: naive
   concatenation is inefficient overall but gets every per-item total right
   because it never merges recipe trees; if the real multi-seed solve
   doesn't reproduce that per-item number exactly, it has re-introduced the
   hand-sum's missed-coupling-edge bug inside the "proper" implementation.
3. **The end deliverable bar is sim-measured, not validator-clean.** The
   canonical EC+AC fixture must build at **zero validation errors** *and*
   be **sim-measured at plan on both targets**, with a long `--warmup` per
   the deep-chain caveat in `docs/status.md`. Validator-clean alone is
   insufficient — [`docs/validator-reporting.md`](validator-reporting.md)'s
   history and #520 (a validator-clean, denser layout that simmed at 0/s)
   are exactly the failure this criterion exists to rule out for the new
   shared-row invariant specifically.
4. **The shared build must beat naive concatenation, or the feature isn't
   paying for itself.** Naive concatenation (two independent solves, placed
   side by side) is the cheap alternative that is always available without
   this RFC and costs ~10% machine overhead on the probe case (Motivation
   table). If the multi-target shared build cannot beat side-by-side
   duplication on area and/or machine count while still clearing kill
   criterion 3 (at-plan on both targets), the added solver/layout complexity
   has no payoff over "run the pipeline twice" — stop and either find the
   win or recommend concatenation as the shipped answer.
5. **N=1 must stay bit-identical.** Every existing single-target call site,
   test, and golden must produce byte-identical output once the
   multi-target entry point lands, by construction (the 11 scalar wrappers
   call the new N-target entry point with a one-element slice — see
   §Solver). If any single-target output changes — one entity, one route,
   one golden hash — stop and fix before proceeding; this mirrors the
   RFC-046/053 "S=1 bit-identical / never-worse" rollout-safety precedent
   and there is no tolerance for drift here, since the entire single-target
   corpus is the regression net this RFC must not disturb.

## Verification plan

Per the layout-engine protocol in
[`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Phase 0 spike is itself the first verification artifact** — its
  observed behavior on the hand-built shared-row `SolverResult` is what
  kill criterion 1 is graded against; record the trace events
  (`GhostSpecFailed`, `TapBridgeUnbridgeable`, `JunctionStrategyAttempt` are
  the ones most likely to fire on an unhandled dual-claim belt) and the
  decoded snapshot, not just an error count.
- **Full e2e suite green** — `cargo test --manifest-path crates/core/Cargo.toml`,
  all non-ignored tests, at every phase boundary.
- **N=1 pinning test** for kill criterion 5 — asserts byte-identical
  `SolverResult`/`LayoutResult` for a representative single-target fixture
  before and after the multi-target entry point lands, in the style of
  `compact_layout_option_is_explicit_and_validated` / RFC-060's
  never-degrades pin.
- **New EC+AC N=2 e2e fixture** added to `crates/core/tests/e2e.rs`,
  gated non-ignored once it clears validation.
- **Per-instance positioned issues** for the new shared-row outflow
  invariant — never a count in a message, per
  [`docs/validator-reporting.md`](validator-reporting.md).
- **Snapshot inspection of the specific shared row** —
  `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ... --nocapture` on the EC+AC
  fixture, decoded per
  [`docs/layout-snapshot-debugger.md`](layout-snapshot-debugger.md),
  entities at the EC row's coordinates inspected directly, not inferred
  from the warning count.
- **Browser eyeball** on the EC+AC URL once the minimal URL extension
  lands — both export belts visibly present and routed to the boundary.
- **Sim harness** on the EC+AC fixture with per-target delivered-rate
  series (once the harness plumbing lands) and a long `--warmup`, for kill
  criterion 3.
- **Clippy + WASM build** green, as a check not a nit.

## Phasing

0. **Decisive spike (layout only, no solver changes).** Hand-construct a
   `SolverResult` with `electronic-circuit` in `external_outputs` *and* as
   an active producer feeding an `advanced-circuit` consumer row; run
   `build_bus_layout` directly. Characterize the shared EC row's observed
   behavior. Evaluates kill criterion 1 before any solver work starts.
1. **Solver N-item generalization.** New `targets: &[(String, f64)]` entry
   point threaded through `solve_netflow_with_options` → `solve_attempt`;
   the three internal sites (RHS assembly, demand-closure seed,
   output-assembly DFS seed) become loops; the 11 existing scalar wrappers
   become one-element-slice callers; `detect_di_couplings` gets the
   target-membership guard. Evaluates kill criterion 2 (copper-cable
   exactness on the probe case) and establishes kill criterion 5's
   construction guarantee.
2. **Layout shared-row handling.** Generalize the dual-purpose-lane
   mechanism (or whatever Phase 0 determined is the real fix) from
   fluid-only to solids; add the new outflow-conservation validator
   invariant. Real evaluation of kill criterion 1 on the actual pipeline
   (Phase 0's spike used a hand-built `SolverResult`; this phase must
   re-clear it on the Phase-1 solver's real output).
3. **Harness per-item verification plumbing.** Per-item checkpoint series
   in `scenario.rs`, per-item verdicts in `report.rs`.
4. **End-to-end EC+AC fixture: validate + sim.** Evaluates kill criteria 3
   and 4 together — zero validation errors, sim-measured at plan on both
   targets, and a direct area/machine-count comparison against naive
   concatenation.
5. **Minimal URL/wasm extension**, only once phases 0–4 clear — sufficient
   to load the EC+AC fixture by URL for manual eyeballing, not a full UI.

Phases 0–1 are the premise test (does a real multi-seed solve fix the
hand-sum bug, and is the shared-row problem tractable without new row
templates); phase 2 is where the harder of the two risks (layout) gets its
real answer; phases 3–4 are the deliverable bar; phase 5 is the minimum
needed to look at the result, not a UI feature.

## Decision log

- *2026-07-31 — RFC opened.* Provenance: 2026-07-31 feasibility research —
  a session-run investigation across solver, layout, export/harness, and
  interface areas (dossier compiled by dedicated research passes over each
  area) plus a hand-run empirical probe comparing naive concatenation
  against the hand-sum shortcut on the canonical `electronic-circuit@10/s` +
  `advanced-circuit@3/s` (from ore, AM2) case. All file:line citations in
  this RFC were re-verified by direct reads against `origin/main` at RFC-open
  time (2026-07-31); the tree moves, so treat line numbers as approximate
  for any reader arriving later. No prior art exists — this is the first RFC
  to address simultaneous multi-target solving or layout; the complexity
  ladder in `docs/status.md` and every existing RFC/issue are single-target
  throughout.

- *2026-07-31 — Phase 0 spike run: kill criterion 1 evaluated, PROCEED
  (with a wider fix shape than the RFC's leading hypothesis).*

  **Setup.** Hand-built a `SolverResult` in a local probe
  (`crates/core/examples/rfc062_phase0_shared_ec_row.rs`, gitignored per
  `CLAUDE.md`'s example-scripts convention — no solver code ran). Two
  `MachineSpec`s on `assembling-machine-2`, flow numbers matching the real
  `electronic-circuit`/`advanced-circuit` recipes at AM2 crafting speed
  (0.75): EC row supplies its own 10/s export target *plus* AC's 6/s
  ingredient draw (16/s combined demand; the row's machine count ceils to
  11, so the row's actual built rate is 16.5/s — visible later in the
  trace as `RowSplit { original_count: 11, ... output_rate=16.5/s }`), AC
  row supplies its 3/s export target from 24 machines. `external_outputs`
  = `[EC@10/s, AC@3/s]`; `external_inputs` covers iron-plate/copper-cable/
  plastic-bar so no upstream smelting chain is needed. `di_couplings` left
  empty by construction — EC has two claims on it (export + AC's
  ingredient draw), so it would not qualify for direct-insertion even
  under a real solve. Ran `build_bus_layout_traced` directly on this
  struct.

  **A confound found and removed before the real test.** Under plain
  `LayoutOptions::default()` (`cell_composition: Candidate`,
  `direct_insertion: Candidate` — both flipped on by RFC-051/RFC-059),
  the decomposition search's RFC-051 cell-composed candidate wins this
  exact EC→AC coupling shape and produces a *different* structure
  entirely: single-row-per-recipe with `corr:`/`row:...:belt-in` segment
  tags, silently using `assembling-machine-3` regardless of the
  `assembling-machine-2` this `SolverResult` specified, and — under an
  earlier belt-tier-capped run of the same fixture — `boundary_outputs`
  contained *only* `advanced-circuit`, with electronic-circuit's own
  target export dropped entirely and no validator error raised for it.
  Kill criterion 1 names the **native** mechanisms specifically
  (`ghost_router.rs` Step 7, `lane_planner.rs` `item_to_consumers`,
  `perimeter_exit_y`) — those live outside the cell-composed path, so the
  probe was re-run with `cell_composition`/`direct_insertion` forced
  `Off` to isolate them. **This confound is itself a real Phase 1/2
  finding, not just a probe artifact**: under today's shipped defaults,
  the EC+AC N=2 fixture may never actually exercise the mechanism this
  RFC is generalizing — the cell-composed candidate wins the race first.
  Phase 2's verification plan needs an explicit check of which candidate
  wins the real EC+AC fixture, not an assumption that it's the native
  path.

  **Observations, native path only (`cell_composition`/`direct_insertion`
  forced `Off`).** Both claims on the shared EC row get built, and they
  physically collide:
  - `LanesPlanned` trace confirms `lane_planner.rs`'s `item_to_consumers`
    claim is real, not hypothetical: a `BusLane` for `electronic-circuit`
    exists with `producer_row=Some(0)`, `extra_producer_rows=[1]` (the EC
    row got split into two row-spans — `RowSplit` at `max_per_row=10,
    output_rate=16.5/s`), `consumer_rows=[2]` (the AC row),
    `x=6`, `tap_off_ys=[19]`.
  - Simultaneously, `placer.rs`'s `is_final` (EC ∈ `external_outputs`)
    drives `output_east=true` for both EC row-spans, so each row's own
    output belt is built running **east** the row's full width
    (`row:electronic-circuit:belt-out`), feeding straight into Step 7's
    export merge (`merger:electronic-circuit`) and out to a real boundary
    exit: `boundary_outputs` lists `electronic-circuit @ (82,29)`,
    undisturbed and physically complete.
  - The lane's tap-off needs a **west**-facing return belt sourcing the
    same row's own output position — observed in the entity dump as
    `ghost:flow:electronic-circuit:6:ret:8` (and `:ret:16` on the second
    row-split), sitting on the exact same `output_belt_y` row as the
    east-flowing merger belts, pointed the opposite direction. The
    junction/crossing solver tries every strategy at the collision seed
    (`(5,1)`: `perpendicular_template`, `sat-surface`, `sat-1ug-native`,
    `sat-2ug-native`, `sat-native`, `eviction`, `sat-1ug-upgrade`,
    `sat-2ug-upgrade`, `sat`, across 4 seed-variant directions) — every
    one comes back `Unsatisfiable`. It gives up
    (`JunctionGrowthCapped { region_tiles: 81, reason: "tile_cap" }`),
    leaving the "ret" tap an orphaned ghost stub with no real source.
    The export merge even tries tunnelling underground specifically to
    dodge the ret-tap tile (`merger:electronic-circuit` goes
    underground at x=25, resurfaces at x=27) and *still* lands head-on
    at the tunnel mouth — the identical pattern repeats independently on
    both EC row-splits (`(25,8)`/`(26,8)` and `(22,16)`/`(23,16)`).
  - **Net physical result**: the export claim wins the tile-level fight
    (it was built first, at row-build time, before `lane_planner` ever
    runs, and its footprint is left undisturbed); the internal-tap claim
    loses completely. Validator: 5 errors (1 `unresolved-junction` at
    `(5,1)`, 4 `belt-junction` HEAD-ON pairs — the two independent
    row-split collisions), 43 warnings — of which **all 24** of AC's
    machines individually report `input-rate-delivery`: `delivers 0.0/s
    but machine needs 0.2/s of electronic-circuit` (total, not partial,
    starvation of the internal-consumer claim; per
    `docs/validator-reporting.md` these are correctly emitted as
    one-positioned-issue-per-instance, not folded into a count — the
    validator did not go quiet here). EC's own ingredient delivery is
    also collaterally degraded (6 `belt-flow-reachability` warnings on
    the row's own pickup belts) — a downstream consequence of the same
    junction cluster failing, not an independent defect. The validator's
    own caveat applies: "orphan ghost belts in this cluster are excluded
    from belt-adjacency checks" — the true blast radius of this specific
    unresolved cluster is plausibly larger than the 5 counted errors.

  **Kill criterion 1 verdict: PROCEED**, but the fix is broader than
  "extend the `perimeter_exit_y` filter" alone. Argued from the observed
  geometry, not speculation: the physical row **templates** need no new
  geometry at all — `templates.rs::output_dir(output_east)` already
  builds identical machine/inserter/belt geometry for either direction,
  just flipping the belt's facing. What's structurally wrong is
  *sequencing and ownership*, not shape:
  1. `is_final` (`placer.rs:~2948`) commits the row's belt direction at
     row-**build** time from "is this item a target" alone, before
     `plan_bus_lanes` ever runs and with no visibility into whether the
     item also has an internal consumer.
  2. `perimeter_exit_y`'s dual-purpose gate is fluid-only by construction
     — confirmed directly in code, not inferred: `lane_planner.rs:198-202`
     builds `surplus_fluid_items` by chaining `surplus_outputs` and
     `external_outputs` then `.filter(|f| f.is_fluid)`, so a solid target
     item never qualifies for the exit-extension this probe needed.
  3. Step 7 (`ghost_router.rs:3439-3444`) unconditionally rebuilds
     `output_items` from `solver_result.external_outputs.iter().filter(|ext|
     !ext.is_fluid ...)` — every non-fluid target gets an independent
     row-level east merge, with no way to skip a row a dual-purpose lane
     already claims.

  Fluids already avoid this exact collision, and the precedent is exact:
  Step 7's own filter is `!ext.is_fluid` — fluid targets get **no**
  row-level east merge at all and rely wholly on the lane's
  `perimeter_exit_y` reaching the boundary (confirmed at
  `ghost_router.rs:3442`). The fix Phase 2 needs is the solid-item mirror
  of that pattern, touching three coordinated existing sites rather than
  inventing a fourth:
  1. `is_final` must also check "does any other row's `spec.inputs`
     consume this item" and, if so, **not** force `output_east=true` —
     treat it as an ordinary internally-routed producer row.
  2. Generalize `surplus_fluid_items` (`lane_planner.rs:198-202`) to also
     cover non-fluid `external_outputs` items that have a real
     `consumer_rows` entry — the dual-purpose-lane shape already modeled
     for fluids applies to solids verbatim once the gate is item-generic.
  3. Step 7's `output_items` (`ghost_router.rs:3439-3444`) must skip any
     target item that got a dual-purpose lane instead of a row-level
     `output_east`, mirroring the existing fluid skip.

  No new physical row/belt/inserter template anywhere in this — every row
  still stamps from the same `SingleInput`/`DualInput`/`TripleInput`
  template family. What changes is which **one** mechanism (row-local
  east belt vs. lane trunk) gets to own a shared item's physical port,
  decided consistently before the row is built, instead of two
  independent mechanisms each building their own claim on the same tile
  and leaving the junction solver to referee an unsatisfiable conflict.

  **For Phase 1/2**: (a) the cell-composition confound above — verify
  which candidate wins the real EC+AC fixture once the solver change
  lands, don't assume native; (b) the three-site fix list above is the
  Phase 2 scope, not a single-field change — budget accordingly; (c) the
  new outflow-conservation validator invariant this RFC's Layout section
  calls for should also assert that a dual-purpose lane's claimed export
  + internal-tap total never exceeds the row's real production rate,
  the solid-item analogue of the same check already implied for fluids.

  **Verification**: `cargo test --manifest-path crates/core/Cargo.toml`
  run clean, one pass, after the probe was added (production code
  untouched — the probe is additive-only and gitignored). No production
  files changed in this phase.
- *2026-07-31 — Phase 1 solver generalization landed: kill criterion 2
  confirmed (exact), kill criterion 5 holds by construction.*

  **Scope.** `crates/core/src/netflow.rs` only, per the RFC's §Solver plan.
  No layout, harness, or interface changes — Phase 2's shared-row fix
  (Phase 0's three-site list) is untouched and still pending.

  **New entry point.** `solve_netflow_multi(targets: &[(String, f64)], ...)`
  and `solve_netflow_multi_with_options(...)` (same trailing args as the
  existing scalar functions, plus `NetflowOptions`) are the new choke
  point. `solve_netflow_with_options` — the inner of the two scalar
  entry points named in the RFC's "3 layered entry points in netflow.rs" —
  now has a two-line body: `solve_netflow_multi_with_options(&[(target_item
  .to_string(), target_rate)], ...)`. `solve_netflow` is untouched (it
  already only calls `solve_netflow_with_options`) and inherits the
  guarantee transitively. Because none of the **8 scalar wrappers in
  `solver.rs`** call anything below `solve_netflow`/
  `solve_netflow_with_options`, **zero lines in `solver.rs` changed** —
  every one of the 11 wrappers the RFC named now bottoms out in the same
  `solve_attempt(targets: &[(String, f64)], ...)` implementation via a
  one-element slice, by construction. This is what makes kill criterion 5
  (N=1 bit-identical) a compile-time guarantee rather than a tested
  coincidence: there is exactly one code path from any scalar wrapper to
  the LP, and the N=1 case is that path's literal special case.

  **The three internal sites**, all in `solve_attempt`:
  1. Target interning now builds `target_order: Vec<usize>` (deduplicated,
     first-seen order) and `target_rate_of: FxHashMap<usize, f64>` (summed
     per unique item) in one pass, replacing the single `target_idx`.
  2. Demand-closure seed: `demanded[idx] = true` for every `idx` in
     `target_order` (was: `demanded[target_idx] = true`).
  3. Output-assembly DFS seed: `target_order.iter().rev().map(|&idx|
     Work::Item(idx)).collect()` (was: `vec![Work::Item(target_idx)]`) —
     reverse-pushed so the first-requested target pops (and is thus
     visited/emitted) first, matching the existing "reverse-push so the
     first producer pops first" convention used elsewhere in the same
     function.
  4. Row RHS (`netflow.rs`, LP constraint assembly): `target_rate_of.get(&i)
     .copied().unwrap_or(0.0)` (was: `if i == target_idx { target_rate }
     else { 0.0 }`). The empty-row-is-an-error check now tests
     `target_rate_of.contains_key(&i)` instead of `i == target_idx`, and
     reports `items.names[i]` (the specific empty row) rather than the
     caller's original string — strictly more precise under N targets,
     and bit-identical to the old message when `targets.len() == 1`.

  **Semantics decision: duplicate target items are summed, not refused.**
  Requesting the same item twice (`&[("electronic-circuit", 4.0),
  ("electronic-circuit", 6.0)]`) collapses to one demand row and one
  `external_outputs` entry at the summed rate (10.0), verified
  bit-identical to requesting `electronic-circuit@10.0` directly
  (`duplicate_target_item_rates_are_summed` in
  `crates/core/tests/solver_multi_target.rs`). Rejected alternative: a
  typed `DuplicateTarget` refusal — summing is strictly less surprising to
  a caller building a target list programmatically (e.g. a future UI that
  lets a user add the same item twice with different rates meaning
  "at least this much") and requires no new error variant threaded through
  every caller.

  **Semantics decision: `external_outputs` and `surplus_outputs` are not
  mutually exclusive.** `external_outputs` now carries one `ItemFlow` per
  unique requested target, in first-seen order, at the (possibly summed)
  demand rate that row's RHS was solved against. `surplus_outputs`
  continues to be computed identically regardless of target membership —
  `o_of(i)` has no target-aware branch anywhere in `solve_attempt`. This
  means a target item CAN legitimately appear in both lists at once: if
  another target's recipe tree forces net production of item `i` above
  `i`'s own requested rate (e.g. a byproduct that is itself a low-rate
  target while a sibling target's demand drives its producer harder), the
  LP satisfies the row via `o[i] > 0` rather than reducing production,
  since reducing production would violate the other target's higher
  draw. Verified this is reachable in principle by direct row-algebra
  (`net_production − consumption + s − o = target_rate` has a legitimate
  `o > 0` solution whenever some *other* column's demand for `i` forces
  gross production above `i`'s own RHS) — not exercised by the KC2 fixture
  itself (EC and AC's coupling runs the other direction: EC's export adds
  to the row's demand rather than exceeding it). **For Phase 2**: a target
  item carrying surplus needs two physical exports of the same item — the
  guaranteed target export plus the surplus export — exactly as a
  non-target byproduct already needs one today; this is a new *instance*
  of dual-purpose-lane provisioning to plan for, not a new mechanism.

  **DI-coupling guard** (`detect_di_couplings`, RFC's "Correctness gap").
  Added a `target_indices: &FxHashSet<usize>` parameter; the very first
  check inside the per-item loop is `if target_indices.contains(&i) {
  continue; }`, before the one-producer/one-consumer/no-surplus/rate-match
  checks run. Verified with a dedicated unit test,
  `netflow::tests::di_coupling_guard_suppresses_target_item_coupling`
  (white-box — needs the private `Column`/`Items` types, so it lives in
  `netflow.rs` itself rather than the integration test file): synthetic
  EC-producer/AC-consumer columns with `x` rates chosen so supply (6.0)
  exactly equals demand (6.0), proving the guard suppresses the coupling
  when EC is in `target_indices` and the same setup couples when it isn't.
  **Finding, not just implementation**: the live KC2 fixture
  (EC@10/s + AC@3/s) does **not** actually exercise this guard — verified
  by temporarily removing the `target_indices` check and re-running the
  probe (`crates/core/examples/rfc062_phase1_kc2_probe.rs`, gitignored).
  Row algebra explains why: for a target item with exactly one producer
  and one consumer and no external supply/surplus, the row constraint
  forces `producer_output − consumer_input = target_rate`, so `supply !=
  demand` in `detect_di_couplings`'s own tolerance check for any nonzero
  target rate — the existing rate-match check already happens to reject
  the pairing before the new guard would need to. The guard is still
  correct and necessary defense-in-depth (a future recipe-graph shape, a
  target requested at an effectively-zero rate, or a different DI
  eligibility refinement could reach the coincidence this guard exists
  to close), and the RFC's own text frames it as a correctness gap to
  close regardless of whether one specific fixture reaches it — but Phase
  2/3 readers should not expect the KC2 fixture's validator/sim behavior
  to visibly change because of this guard; its payoff is defensive.

  **Typed-refusal review.** The acyclic-fallback / oil-path-physical retry
  loop (now `solve_netflow_multi_with_options`) needed no logic changes —
  confirmed target-count-agnostic exactly as the RFC predicted, since every
  branch inspects `r.machines`/`r.surplus_outputs` post-solve. Error
  `target` fields that used to read `target_item.to_string()` now read
  `target_label(targets)` (joins every requested item name;
  bit-identical to the old string when `targets.len() == 1`). Verified the
  machine-incompatibility typed-refusal path still surfaces correctly under
  N=2 with one satisfiable and one unsatisfiable target
  (`multi_target_incompatible_machine_error_not_masked_by_other_target`):
  AM1-pinned `advanced-circuit` alongside a perfectly solvable
  `iron-gear-wheel@5` still returns `SolverError::IncompatibleMachine`,
  not a silent partial success. Did not construct a dedicated multi-target
  cycle-refusal test beyond this — the retry loop's cycle-handling code is
  unchanged (not just unchanged-in-effect but literally the same lines),
  and constructing a deterministic multi-target cycle fixture is
  materially more fragile than the machine-incompatibility case for the
  same evidentiary value; flagging as an explicit scope stop rather than a
  silent gap.

  **Kill criterion 2 — measured, exact.** Canonical probe
  (`electronic-circuit@10/s` + `advanced-circuit@3/s`, from ore, AM2,
  `RecipeScope::Free`), `kc2_ec_ac_shared_copper_cable_exact` in
  `crates/core/tests/solver_multi_target.rs`:

  | Item | Multi-seed LP (this phase) | Naive-concatenation total (Motivation table) |
  |---|---:|---:|
  | copper-cable | **20.000000** | 20.0 |
  | electronic-circuit | **10.666667** (16/1.5 exact) | — (10.667 in the RFC's rounded table) |
  | iron-plate | **25.600000** | 25.6 |
  | copper-plate | **48.000000** | 48.0 |

  All four asserted to within `1e-9` of the closed-form expected value
  (derived independently from crafting-speed/energy arithmetic, not copied
  from the RFC table, then cross-checked against it) — none within 1e-9 of
  the hand-sum shortcut's wrong numbers (16.0 copper-cable machines).
  `advanced-circuit` itself solves to 24.0 machines and `dependency_order`
  places `electronic-circuit` before `advanced-circuit`
  (`kc2_dependency_order_ec_before_ac`), confirming the shared upstream is
  genuinely deduplicated rather than solved twice and concatenated.

  **Kill criterion 5 — pinned, not just argued.** Beyond the construction
  guarantee above,
  `n1_equivalence_multi_matches_scalar`/`n1_equivalence_holds_on_multi_hop_target`
  assert full field-level bit-identity (`f64::to_bits()` on every rate/
  count field, not approximate equality) between `solve_netflow_multi`
  called with a one-element slice and the existing scalar `solve_netflow`,
  on both a single-recipe target and a multi-hop coupled target
  (`advanced-circuit` from ore). Combined with the full existing suite
  passing at zero golden churn (below), this is the systemic proof the RFC
  asked for.

  **Verification.** `cargo test --manifest-path crates/core/Cargo.toml` —
  one clean invocation, all non-ignored tests, 0 failures: lib 920 passed
  (919 pre-existing + 1 new DI-guard unit test), `solver_netflow_parity`
  11 passed (unchanged), `solver_multi_target` 7 passed (new), every other
  suite unchanged. `cargo clippy -p spaghettio_core -- -D warnings` (the
  exact pre-commit hook invocation) clean. `cargo build -p spaghettio_wasm`
  clean (native-target sanity check only — the wasm bindings' public
  surface was not touched, so a full `wasm-pack` rebuild was judged
  unnecessary for a solver-internals-only change; Phase 5 is where the
  wasm/URL surface actually changes and gets the real wasm-pack + browser
  verification pass). No `.fls` snapshot or golden file changed.

- *2026-07-31 — Phase 2 shared-row layout fix landed: kill criterion 1
  re-cleared on the real Phase-1 solver output, the new outflow-
  conservation validator invariant shipped two-sided, one confirmed
  pre-existing gap found and documented (not fixed).*

  **Scope.** `crates/core/src/bus/placer.rs`, `lane_planner.rs`,
  `ghost_router.rs` (the three sites Phase 0's decision log named) plus
  the new validator check in `crates/core/src/validate/mod.rs`. No
  solver changes (Phase 1 untouched). No harness/interface changes
  (Phases 3/5 still pending).

  **The three-site fix, exactly as Phase 0 specified.**
  1. `placer.rs::place_rows` — a new `internally_consumed_items` set
     (every non-fluid item appearing in any non-voider `MachineSpec`'s
     `inputs`, computed once per call) is subtracted from `is_final`'s
     final-item test: `is_final = spec.outputs.iter().any(|o| !o.is_fluid
     && final_items.contains(item) && !internally_consumed_items.contains
     (item))`. Voiders are excluded from the set deliberately — their draw
     is bus-invisible by design (mirrors `lane_planner`'s own voider
     exclusion) and folding them in would wrongly flip `is_final` off for
     a target whose only consumer is a voider, breaking the existing
     Step-7c voider mechanism. A normal (non-self-loop) `MachineSpec`
     never lists the same item in both `inputs` and `outputs`
     (self-recirculation uses `self_loop` instead), so the set can safely
     include every machine's inputs, including the producer's own row's,
     without a row reading itself as its own internal consumer.
  2. `lane_planner.rs::plan_bus_lanes` — a new `solid_target_items` set
     (non-fluid `external_outputs` items) gates a second `perimeter_exit_y`
     pass, separate from the existing fluid one: a lane qualifies when
     `!lane.is_fluid && !lane.consumer_rows.is_empty() &&
     solid_target_items.contains(item)`. Deliberately narrower than the
     fluid gate (which also covers zero-consumer fluid targets, since
     Step 7 never builds a row-level merge for fluids at all) — a
     zero-consumer solid target keeps its existing `is_final`/
     `output_east` path untouched, which is what makes kill criterion 5
     (N=1 bit-identical) hold by construction rather than by testing. No
     exit-y staggering (unlike the fluid case's F1 pipe-merge avoidance):
     solid belts at the same y on different columns don't interact.
  3. `ghost_router.rs` Step 7 — `output_items` additionally filters out
     any item present in a new `dual_purpose_solid_items` set (derived
     the same way as (2), from `lanes` directly), mirroring the existing
     `!ext.is_fluid` fluid skip.

  **A fourth site Phase 0's design text didn't name but the physical
  mechanism requires**: `ghost_router.rs`'s SOLID trunk-stamping loop
  (the one before fluid Step 3.6, unconditional on `is_fluid`) computed
  `end_y` from `tap_off_ys`/`producer_ys`/`balancer_y` only — it had no
  knowledge of `perimeter_exit_y` at all, because before this phase no
  solid lane ever set it. Extended `end_y` to
  `end_y.max(lane.perimeter_exit_y.unwrap_or(end_y))`, and added a
  `surplus_exits` record (mirroring the fluid Step 3.6 code exactly) so
  `check_stranded_byproducts` and the new outflow check can
  entity-cross-check the claim. Without this site, `perimeter_exit_y`
  being set on a solid lane would be inert — the trunk would stop at the
  last tap and the flow would strand exactly at the boundary the field
  claims to reach. Documented here because a reader diffing this PR
  against Phase 0's "3-site fix" list should not conclude a site was
  missed; it's the same fix, one level more concrete than Phase 0's
  design-time text could get without running the pipeline.

  **The new validator invariant — shipped two-sided, not one-sided.**
  `check_shared_row_outflow_conservation` (`validate/mod.rs`), wired into
  `validate()`'s dispatch, gated on `solver.is_some()` like
  `check_stranded_byproducts`. Operates purely on `SolverResult` (target
  rate, tap demand from every non-voider machine's `inputs`, surplus
  rate, all summed per item using the CEILED per-`MachineSpec` machine
  count — the same rounding `place_rows` applies, so `production` matches
  what's actually built, not the LP's fractional demand) plus
  entity-cross-checked `boundary_outputs`/`surplus_exits` for the
  physical-realization check. A "shared row" requires **at least two** of
  `{target > 0, taps > 0, surplus > 0}` to be live — deliberately
  broader than just "target + taps" (the EC+AC shape) so it also covers
  "target + surplus" (the adversarial U-235/U-238 shape below); a row
  with only one live claim is never flagged, since the solver's own flow
  conservation already guarantees it's correct and this check exists to
  catch a LAYOUT-side failure to honor that conservation, not to
  re-litigate the LP.
  - **Over-claim** (`shared-row-outflow-overclaim`, error): `target +
    taps + surplus > production`. The RFC's primary ask. One positioned
    issue per violating item, positioned at a real producer-machine
    entity (rule 4, `docs/validator-reporting.md` — never `(0,0)` unless
    genuinely no matching entity exists), carrying `IssueDetail{
    delivered: production, needed: claimed }`.
  - **Under-claim** (`shared-row-outflow-underclaim`, error): the item
    has a live target AND at least one other live claim, but no physical
    export record backed by a real entity exists at all (checked against
    both `boundary_outputs` and `surplus_exits`, matching
    `check_stranded_byproducts`'s and `check_boundary_record_integrity`'s
    own standard). **Decision, not assumed**: the RFC's task brief asked
    whether the invariant should be two-sided, specifically to catch
    Phase 0's own "dropped export under the cell-composed candidate"
    observation. Traced why the existing checks miss it:
    `check_belt_network_topology`'s output-network check seeds a BFS from
    `output_inserter_belt_tiles` and returns immediately when
    `belt_starts.is_empty()` — a fully-dropped export (zero output
    inserters found for the item at all) produces an empty `belt_starts`
    and the check silently returns without emitting anything, the same
    "union hides an empty case" shape as failure #10 in
    `docs/validator-reporting.md`'s table. `check_output_belt_coverage`
    only asks "does this machine have SOME output belt", not "does this
    item's production reach the layout boundary" — a fused cell where EC
    feeds AC directly still has an output belt, so it passes too. Neither
    existing check would have caught Phase 0's dropped-export finding.
    Added the under-claim direction rather than leaving it as a
    documented gap, at low marginal cost (the same entity-cross-check
    machinery `check_stranded_byproducts` already uses, reused verbatim).
  - Five unit tests pin the check's own logic in isolation
    (`crates/core/src/validate/mod.rs`, `check_shared_row_outflow_*`):
    quiet within capacity, over-claim fires with exact `IssueDetail`
    numbers, under-claim fires on a missing record, under-claim fires
    even when a `surplus_exits` ledger entry exists with no backing
    entity (the "ledger without the entity" case rule 4 exists for), and
    a non-shared row (fewer than 2 live claims) is never flagged
    regardless of numbers.

  **Regression fixtures** (`crates/core/tests/layout_multi_target.rs`,
  new file — the RFC's plan said `e2e.rs`, but `e2e.rs`'s `RunParams`
  harness is built around `solver::solve*`'s scalar `target_item`/
  `target_rate` and cannot drive `solve_netflow_multi`; Phase 1 hit the
  same shape and made the same call with `solver_multi_target.rs`, so
  this mirrors established precedent rather than inventing a new one).

  1. **`ec_ac_shared_row_native_mechanism_zero_errors`** — the mechanism
     fixture, `cell_composition`/`direct_insertion` forced `Off` per the
     task brief, to guarantee the native 3(+1)-site fix (not a
     cell-composed candidate) is what's under test. Zero validation
     errors. Zero `shared-row-outflow-{overclaim,underclaim}` issues.
     Zero `input-rate-delivery` issues mentioning electronic-circuit
     (Phase 0's exact original symptom — AC's machines starved). The EC
     lane's `LanesPlanned` trace entry has non-empty `consumer_rows` and
     `tap_off_ys` (AC's tap-off is real, not hypothetical);
     `layout.surplus_exits` has an electronic-circuit entry backed by a
     real belt entity at that exact tile (the perimeter exit is real, not
     a ledger claim); AC's real machine-entity count in the layout
     matches the solver's ceiled AC machine count exactly (AC is actually
     built, not just claimed). A companion example script
     (`crates/core/examples/rfc062_phase2_ec_ac_snapshot.rs`, gitignored,
     writes a `.fls` snapshot to `$TMPDIR/spaghettio_rfc062_phase2/`) was
     used to eyeball the exact tiles: electronic-circuit's row-out belt
     at (23-32,76) now runs **West** (was East pre-fix), the trunk column
     x=2 collects from three row-splits (y=72/80/83 — RowSplit ceiled 16/
     1.5=10.67 target+tap demand to more machines than one row holds),
     taps east into AC's `row:advanced-circuit:belt-in:electronic-circuit`
     at y=87-89 (24 long-handed inserters, matching AC's 24-machine
     count), and the SAME trunk continues south past the tap to the
     perimeter exit — `boundary_outputs` for electronic-circuit is empty
     (correctly: Step 7's row-level merge was skipped for this item;
     `surplus_exits` owns the physical claim instead).

     **Residual account, corrected (RFC-062 Phase 2 review, F1) — what
     this entry originally said, what was actually true, and why the gap
     went unnoticed.** Originally recorded here: "Residual: 4
     `input-rate-delivery` warnings (not errors) on copper-cable... a
     pre-existing lane-balancing gap." **That was incomplete, not
     wrong**: the fixture's own `eprintln!` printed "0 errors, 28
     warnings" (visible in the original PR's test output), but the
     write-up only ever inspected the `input-rate-delivery` category
     subset (4 hits, all copper-cable) and never accounted for the other
     24 — exactly the shape `docs/validator-reporting.md` rule 6 warns
     about ("count, don't sample" — the write-up sampled one category and
     silently dropped the rest). The 24 were `belt-flow-reachability`
     warnings, one per AC machine ("items cannot leave"), caused by a
     real bug: `lane_planner.rs`'s solid dual-purpose-lane exit was
     `perimeter_exit_y = Some(total_height)`, but `total_height` is an
     EXCLUSIVE bound (rows occupy `[0, total_height)`) — the exit landed
     one row PAST the layout's real bottom edge (originally (2,96) on a
     96-row layout). `check_belt_flow_reachability`'s belt-derived
     boundary (`max_by` over every belt tile it can see) shifted down by
     one row to match, demoting every legitimate y = total_height − 1
     export tail — AC's own row among them — from "boundary" to
     "interior". The fluid dual-purpose lane's identical `total_height +
     exit_offset` convention never surfaces this: a fluid exit is a pipe,
     invisible to this belt-only check's `on_boundary`. Fixed:
     `perimeter_exit_y = Some(total_height - 1)` (now (2,95) on this
     fixture). **Corrected, true account**: 0 errors, 4 warnings — all 4
     `input-rate-delivery` on copper-cable, exactly as first described,
     now with the belt-flow-reachability wave actually gone rather than
     merely unmentioned. Copper-cable's own residual (AC draws 2
     cable/craft directly, on top of EC's own cable draw — the RFC's own
     Motivation table names this second coupling edge) is unchanged: an
     ordinary multi-consumer lane, not itself an external target so
     `solid_target_items` never gates it, out of scope here. The
     mechanism fixture now asserts `belt-flow-reachability` warnings are
     zero explicitly, not just folded into a total.
  2. **`ec_ac_default_options_candidate_choice`** — documents what the
     shipped-default engine (`cell_composition`/`direct_insertion` both
     `Candidate`) actually does with the real Phase-1 solver output.
     **Result: the confound does not reproduce.** `DecompositionChosen`
     names `"native"` as the winner, zero validation errors, and
     electronic-circuit's export claim is present. Phase 0's own
     confound was observed on a **hand-built** `SolverResult`
     (`examples/rfc062_phase0_shared_ec_row.rs`) with `assembling-
     machine-2` hardcoded and specific flow numbers chosen by hand; the
     REAL Phase-1 solve for this exact recipe pair apparently doesn't
     land in the same region of the decomposition search's scoring
     function that made the cell-composed candidate win in Phase 0's
     probe. This is verified for the ONE canonical fixture, not proven
     in general — a different multi-target shape could still exercise
     the cell-composed path first. The test pins `DecompositionChosen`'s
     name so a future engine change that flips the winner for this exact
     shape is a visible test diff, not silent drift.
  3. **`u235_u238_target_and_surplus_overlap`** — the adversarial
     reviewer's case, U-235@0.1 + U-238@0.05 kovarex excluded. Confirmed
     the solver produces the expected large U-238 surplus (14.136/s,
     matching the reviewer's 14.13/s hand-derivation). **Found a real,
     pre-existing bug, distinct from the EC+AC dual-purpose-lane
     mechanism**: neither uranium item has an internal consumer (kovarex
     excluded), so `lane_planner`'s `solid_target_items` gate never
     applies to either — this fixture exercises the OLDER D2a/D2b
     solid-surplus-secondary-belt path (`docs/rfc-fulgora-scrap.md`),
     asked for the first time to treat BOTH of a D2b row's distinct solid
     outputs (uranium-235 primary, uranium-238 secondary via
     `RowSpan::secondary_output_belt`) as external targets simultaneously
     — a shape only reachable now that Phase 1's solver can request two
     targets at once. `ghost_router.rs` Step 7's per-item merge
     unconditionally sources `output_ys` from `row_spans[ri].
     output_belt_y` (the row's PRIMARY belt) for every item in
     `output_items`, with no branch for `secondary_output_belt` — so
     uranium-238 as a Step-7 target merges from uranium-235's belt.
     Measured result: 12 real validation errors (8 `entity-overlap`, 4
     `belt-item-isolation` — "belt at (134,Y) carries uranium-235 but
     feeds into (135,Y) which carries uranium-238", at all four of the
     row's split sub-rows). uranium-235's own export (existing,
     unaffected path) stays clean. **Decision**: not fixed in this
     phase — Step 7's `output_ys` needs to become per-item aware of
     `secondary_output_belt`/`sorted_output_belts` (the D2b/D3
     multi-output-belt machinery), a change to a different, older
     mechanism than this phase's 3(+1)-site fix, deserving its own
     investigation rather than a bolted-on patch under review-pressure.
     Made LOUD per the task brief: the test **asserts** (not just logs)
     both the ledger-level "export claim present" state (still true —
     `exported()`'s own check is too weak to catch this class of
     corruption, itself a finding) AND the specific error categories/
     non-empty counts, so a future fix to Step 7's D2b targeting is
     forced to update this test rather than the gap silently
     disappearing or reappearing unnoticed. Tracked as a followup:
     **Step 7 must resolve a target item's per-row belt-y from
     `RowSpan::output_belt_y_for(item)`** (the same helper
     `lane_planner` already uses for exactly this purpose, per its own
     doc comment) instead of the bare `output_belt_y` field, for every
     row in `output_rows`, not just the D2b case specifically — this is
     the general fix shape, not a uranium-specific patch.

  **N=1 regression.** Full suite run clean before AND after adding the
  new tests: `cargo test --manifest-path crates/core/Cargo.toml` — 926
  lib tests passed (920 Phase-1 baseline + 6 new
  `check_shared_row_outflow_*` unit tests, one added in the review fix
  round below), 0 failures, 3 ignored (unchanged); every existing
  integration-test file's pass count unchanged; no `.fls` snapshot or
  golden file touched or regenerated.
  `is_final`'s new `internally_consumed_items` check only changes
  behavior when a row's own output item is BOTH in `final_items` AND
  appears in some non-voider machine's `inputs` — no existing
  single-target fixture in the corpus has ever hit that intersection (a
  single target being simultaneously consumed internally requires a
  second demand edge on the target item that only exists once a SECOND
  target's recipe tree creates one), so this is a construction argument
  backed by the full suite staying byte-for-byte green, not just an
  absence of observed diffs.

  **Verification.** `cargo test --manifest-path crates/core/Cargo.toml`,
  one clean invocation as above. `cargo clippy -p spaghettio_core --
  -D warnings` (the exact pre-commit hook invocation) clean. `cargo build
  -p spaghettio_wasm --manifest-path crates/wasm-bindings/Cargo.toml`
  clean (native-target sanity check; the wasm public surface is
  untouched by this phase — Phase 5 gets the real wasm-pack + browser
  pass). **Browser eyeball: N/A, explicitly** — this phase changes only
  multi-target layout paths; no single-target visual change is expected
  or possible (kill criterion 5's construction guarantee), and the
  minimal URL/wasm extension needed to even LOAD an N=2 fixture in the
  browser is Phase 5's scope, not yet landed. Snapshot inspection
  performed via a gitignored example script rather than the e2e harness'
  built-in dumper (which `layout_multi_target.rs`, being outside
  `e2e.rs`, doesn't have wired up) — entities inspected directly at the
  EC row's actual coordinates (not Phase 0's, which came from a
  different hand-built fixture and different row placement), confirming
  the West-flowing row-out belt and the tap-then-continue trunk shape
  described above.

  **For Phase 3+**: (a) the Step-7/D2b `output_belt_y_for` followup
  above, tracked but not scheduled; (b) the copper-cable lane-balancing
  warning on the EC+AC fixture is a residual, not investigated further
  here — worth a look before Phase 4's sim-harness pass, since a
  warning-level under-delivery could still measure as a real rate gap;
  (c) Phase 3 (harness per-item verification plumbing) is unblocked by
  this phase — the EC+AC fixture now builds validator-clean under the
  native mechanism, which is Phase 3/4's prerequisite.

- *2026-08-01 — Phase 2 adversarial review fix round: one blocking
  finding fixed (F1), the over-claim direction gained a severity split
  (F2), two teeth items added (F3, F4), two reviewer observations
  recorded.*

  **F1 (blocking, fixed).** `lane_planner.rs`'s solid dual-purpose-lane
  exit set `perimeter_exit_y = Some(total_height)`. `total_height` is an
  EXCLUSIVE bound (rows occupy `[0, total_height)`, the same convention
  `RowSpan::y_end` uses) — the exit landed one row PAST the layout's real
  bottom edge. `check_belt_flow_reachability`'s belt-derived boundary
  (`max_by` over every belt tile) shifted to match, demoting every
  legitimate `y = total_height − 1` export tail from "boundary" to
  "interior": 24 false `belt-flow-reachability` warnings, one per AC
  machine, on the canonical fixture (0 errors either way — this was
  always warning-severity, never a build blocker, which is exactly why
  it survived the original `errors.is_empty()` assertion). The fluid
  dual-purpose lane's identical `total_height + exit_offset` convention
  never surfaces this: a fluid exit is a pipe, invisible to this belt-
  only check. Fixed: `Some(total_height - 1)`. Verified: the mechanism
  fixture's warning count drops from 28 to 4 (the 4 copper-cable
  `input-rate-delivery` warnings, unchanged and out of scope), and the
  fixture now asserts `belt-flow-reachability` warnings are zero
  explicitly rather than folding them into an unexamined total — see the
  corrected residual account inline above (Phase 2's original entry) for
  what went wrong the first time and why.

  **F2 — over-claim severity split, decided and documented.** Naively
  comparing BUILT totals alone (every `MachineSpec`'s machine count
  independently ceiled) hard-errors on legitimate ceil-slack contention:
  reviewer-measured on EC@10.4/s + AC@3.05/s, claimed (10.4 target + 6.25
  built taps) = 16.65 > produced (built) 16.5 — the sole issue on an
  otherwise-clean layout. The producer's and the tap's consumer are two
  INDEPENDENT `MachineSpec`s, each ceiled on its own; nothing guarantees
  their rounding slack cancels, so a built-only comparison conflates
  "genuine demand overrun" with "two roundings that both happened to
  round up." Decision: `check_shared_row_outflow_conservation` now also
  computes the same totals from the LP's RAW (un-ceiled)
  `MachineSpec::count` — the net-flow LP's own row-conservation identity
  guarantees `target_plan + taps_plan + surplus_plan ≈ production_plan`
  unless something is genuinely wrong at the plan level, not just the
  build level. Only a PLAN-level over-claim (`claimed_plan >
  production_plan`) is `Severity::Error`; a BUILT-only over-claim (plan
  balances, built doesn't) is `Severity::Warning` with a distinct
  message — still surfaced (the row has zero headroom for its exact
  built machine-count pairing, worth knowing about), just not a claim
  that a consumer or the export will actually starve. `target`/`surplus`
  needed no plan/built split of their own — the solver never ceils a
  target rate or a surplus remainder, so those two terms are already
  plan-precision on both sides of the comparison; only the tap term
  (drawn from a `MachineSpec::count` that gets independently ceiled by
  the placer) does. New unit test
  `shared_row_outflow_conservation_overclaim_is_warning_when_only_ceil_
  slack` (`validate/mod.rs`) reproduces the reviewer's shape with a
  minimal synthetic pair (producer count 10.0 exact, consumer count
  6.001 — `ceil(6.001) = 7`) and asserts `Severity::Warning`; the
  existing `shared_row_outflow_conservation_flags_overclaim` test
  (integer machine counts, no ceil slack) now documents explicitly that
  it's pinning the OTHER branch (`Severity::Error`) and must stay there.

  **F3 — site 3 (ghost_router Step 7's `output_items` skip) was
  untested; teeth added.** Reverting the skip alone (leaving the
  dual-purpose lane's own routing intact) doesn't change the mechanism
  fixture's error/warning counts by itself — Phase 0's original collision
  came from site 1 (`is_final` forcing `output_east=true`), which site 3
  alone can't undo, so a count-based assertion would have stayed green
  under a site-3 revert. Added a direct assertion instead: no entity in
  the mechanism fixture carries `segment_id ==
  Some("merger:electronic-circuit")` — `output_merger::merge_output_rows`
  always stamps that exact segment tag, so its absence is site 3's real
  signature. Noted alongside it, for a future reader: reverting site 3
  also flips `output_items.len()` from 1 (`["advanced-circuit"]`) to 2
  (`["electronic-circuit", "advanced-circuit"]`), which changes Step 7's
  `merge_x_cursor` start from the single-item convention (0) to the
  multi-item one (`row_spans.iter().map(row_width).max() + 1`) — every
  merger entity's x-position would shift east, a second, independent
  signal a reverted site 3 would trip.

  **F4 — fixture 2's winner assertion pinned to identity, not just
  presence.** `ec_ac_default_options_candidate_choice` previously
  asserted only `winner.is_some()` while the decision log's own text
  claimed the test "pins the winner's identity" — the assertion didn't
  back that claim. Changed to `assert_eq!(winner.as_deref(),
  Some("native"), ...)`. Noted in the test and here: `#553`'s DI-scoring
  changes (concurrent work in this session) could legitimately flip which
  candidate wins this shape later — if it does, this assertion failing is
  the intended visibility, not a false alarm; update it deliberately and
  re-verify the CONFIRMED/else branch still matches reality rather than
  loosening the assertion back to `is_some()`.

  **Reviewer observations, recorded verbatim-in-spirit.**
  - The under-claim direction's "physically exported" standard is a
    **record backed by an entity at the recorded tile**, not a flow or
    rate measurement. A belt can satisfy this check while under-
    delivering, or — per `docs/validator-reporting.md`'s own history —
    being disconnected one tile further along a chain the check never
    walks. The sim harness (Phase 3/4) remains the real bar for "the
    claimed rate actually moves in-game"; this check only closes the
    "nothing there at all" gap Phase 0 observed, nothing stronger.
  - **Untested shape, named as a Phase 3 fixture candidate**: target +
    taps + surplus all live on ONE item at once, specifically U-238 as a
    target under multi-target with kovarex excluded, where U-238 is
    ALSO internally consumed (so it drives Step 7b's D2a merge from a
    row that `is_final`'s new gate has flipped west, per this phase's own
    site-1 change) — a genuinely three-way collision this phase's two
    adversarial fixtures each exercise only two of the three axes of
    (`ec_ac_shared_row_native_mechanism_zero_errors`: target+taps, no
    surplus; `u235_u238_target_and_surplus_overlap`: target+surplus, no
    taps). Not constructed this round — the recipe-graph shape needed
    (an item that is simultaneously a D2b secondary output, a Step-7b
    surplus source, AND a `solid_target_items`-gated dual-purpose-lane
    consumer) doesn't exist in the current recipe DB without a
    synthetic/hand-built `SolverResult`, and building one well enough to
    be a trustworthy regression net (not just a fragile one-off) is
    real, Phase-3-sized work, not a fix-round add-on.

  **Verification.** `cargo test --manifest-path crates/core/Cargo.toml`
  — 926 lib tests passed (925 + 1 new: the F2 ceil-slack-warning unit
  test), 0 failures, 3 ignored (unchanged); every other suite unchanged;
  `layout_multi_target`'s 3 tests all green with the new F1/F3/F4
  assertions in place; no `.fls` snapshot or golden file touched.
  `cargo clippy -p spaghettio_core -- -D warnings` clean. As predicted,
  the mechanism fixture's warning count is the only behavioral diff from
  this round (28 → 4) — no error-count change, no other fixture's
  pass/fail flipped.

- *2026-08-01 — Phase 3 landed: wasm multi-target entry + boundary
  validation, minimal URL extension, harness per-item checkpoint series.
  No layout/solver changes (Phases 1/2 untouched).*

  **Scope.** `crates/core/src/solver.rs`, `crates/wasm-bindings/`,
  `web/src/{state,engine}.ts` + `web/src/ui/sidebar.ts` +
  `web/src/workers/engine.worker.ts`, `crates/sim-harness/src/{scenario,
  report}.rs`, `crates/core/examples/sim_export.rs`. Everything the RFC's
  §Interface and §Export/sim-harness sections scoped as "small,
  deliberately minimal."

  **wasm entry (§Interface).** New `solver.rs` wrapper
  `solve_multi_with_palette_exclusions_quality_and_modules(targets: &[(String,
  f64)], ...)` — the multi-target counterpart of
  `solve_with_palette_exclusions_quality_and_modules`, delegating to
  `netflow::solve_netflow_multi_with_options` with the FULL targets slice
  (not a one-element wrapper of the scalar path — this is a new, direct
  caller of Phase 1's multi entry point at the palette/quality/modules
  richness the wasm boundary needs). Pinned bit-identical to the scalar
  function at N=1 by construction
  (`n1_equivalence_holds_at_solver_rs_multi_wrapper`,
  `solver_multi_target.rs`), same proof shape as Phase 1's own N=1
  guarantee.

  `wasm-bindings/src/lib.rs` gets `solve_multi(targets: Vec<Target>,
  available_inputs, palette, default_machine, quality, modules) ->
  Result<SolverResult, JsError>`, mirroring `solve_with_palette`'s
  signature with `target_item`/`target_rate` replaced by `targets`.
  `Target { item: String, rate: f64 }` derives `tsify_next::Tsify`
  directly in the wasm-bindings crate (a new `tsify-next` dependency
  there, same version/features as core's own wasm-gated models) — a plain
  JS `{item, rate}` object array crosses the boundary natively
  (`Vec<Target>` as a direct wasm-bindgen parameter, confirmed working via
  a real `wasm-pack build`, not just the native sanity check Phase 1 used
  — the wasm public surface DOES change this phase). Generated TS:
  `solve_multi(targets: Target[], ...)`. `layout`/`layout_traced`/
  `layout_streaming`/`export_blueprint` needed zero changes — verified,
  not reworked, per the RFC's own claim that they're already
  `SolverResult`-agnostic (confirmed by tracing every call site: none
  branch on `external_outputs.len()`).

  **Boundary validation (the deferred item).** `validate_targets(&[Target])`
  in `wasm-bindings/src/lib.rs`, called at the top of `solve_multi` before
  any solve work: rejects an empty target list, a non-positive/
  non-finite rate, or an item absent from `recipe_db::all_producible_items()`,
  each with a `[INVALID_TARGETS]`-prefixed `JsError` (same marker
  convention as `solver::INCOMPATIBLE_MACHINE_PREFIX`). **The split is
  deliberate and documented at both ends**: `solve_multi_with_palette_
  exclusions_quality_and_modules`'s own doc comment states it does NOT
  validate (matching every other `solver.rs` entry point's posture), and
  `validate_targets`'s doc comment explains why the wasm boundary is the
  right layer instead — an empty `targets` slice quietly solves to an
  empty plan today (no LP constraints, no error); a non-positive rate
  produces a degenerate-but-solvable row; and a genuinely unknown item is
  eventually caught deep inside `solve_attempt` as a generic `LpFailed`
  ("no producer, no external supply, no surplus sink") only after the
  full candidate-recipe LP is built — correct, but late and non-specific
  versus a fast, typed, pre-flight message the ONE caller that needs it
  (a user-facing UI) can render immediately. Internal callers (this
  crate's own `solve`/`solve_with_palette`, `sim_export.rs --multi`) are
  trusted call sites and pay no extra validation cost.

  **Minimal URL extension (§Interface, "no full sidebar UI").**
  `web/src/state.ts`'s `FormState` gains `targets: {item, rate}[] | null`
  — `null` (the overwhelming common case) is single-target, `item`/`rate`
  authoritative exactly as before. Encoded as `;`-separated `item:rate`
  pairs: `tg=` extras key in the hash form, `targets=` in the legacy
  query form (no short-coding — item slugs are hyphen-only so never
  collide with either separator, and short-coding a field the editing UI
  never writes wasn't judged worth the added surface for a "minimal"
  extension). The writer always keeps `item`/`rate` (the primary slot)
  synced to `targets[0]`, so a multi-target URL degrades gracefully: a
  client that ignores `tg=`/`targets=` entirely still renders a working
  single-target layout for the first requested item, rather than falling
  back to defaults. Round-trip tested
  (`web/src/state.test.ts`, "multi-target extras" describe block):
  encode → decode identity, malformed values (`bogus-no-rate`, negative
  rate) reject to `null` rather than a partial list (same "whole field
  invalid" posture as the existing short-code parser), single-target
  state omits `tg=` entirely.

  `web/src/ui/sidebar.ts` — the editing UI's ONE touch point:
  `writeUrlState` always writes `targets: null` (the sidebar's controls
  are single-target-only, per RFC scope), and a new `runSolveMulti`
  function (parallel to `runSolve`, calling `engine.solveMulti` instead
  of `engine.solve`) fires exactly once, at page load, only when
  `urlState.targets.length > 1` — nothing else calls it, so every
  subsequent edit through the form falls back to the normal single-target
  path, which is the documented contract ("editing UI stays
  single-target"). `engine.ts` + `workers/engine.worker.ts` get the
  `solveMulti` RPC plumbing (`method: "solveMulti"` dispatching to the
  new `solve_multi` wasm export), mirroring the existing `solve`/
  `solve_with_palette` RPC shape exactly.

  **Harness per-item checkpoint series (generalizing the 2026-07-24
  first-target-only fix).** `scenario.rs::build_control_lua`: the single
  `local TARGET = "..."` global becomes `local TARGETS = {...}` (every
  requested target, in manifest order) plus `local TARGET = TARGETS[1] or
  ""` — window-closing/convergence timing is UNCHANGED (still keyed off
  the first target only, exactly as before this phase: a run's length
  and stability gating are not something this phase touches). A new Lua
  helper, `checkpoint_items()`, records `{item -> {produced, delivered}}`
  for every target at each checkpoint-window close, additive alongside
  the existing flat `produced`/`delivered` scalar fields (which stay
  first-target-only, unchanged). `report.rs::compute` now reads each
  target's OWN per-item series first (`checkpoint_item_series` +
  `rate_over_checkpoints`), falling back to the old flat-scalar path only
  for the first target on an older `raw_result` with no `items` key, and
  to the pre-Phase-3 produced-only sample-series path for a non-first
  target on such an old result — three-tier fallback chosen specifically
  so every existing fixture/goldenwas unaffected: the full harness test
  suite (65 tests) passes byte-for-byte unchanged, and a new test
  (`second_target_gets_its_own_delivered_rate_verdict`) proves the fix
  directly — a synthetic second target with a deliberate small
  under-delivery (2.96/s measured vs 3.0/s planned) now gets
  `measured_delivered_rate: Some(2.96)` and a verdict computed FROM that
  delivered rate, where before this phase it would have been `None`
  (produced-only fallback, the exact gap the Phase 2 review named). This
  is the mechanism KC3 below depends on: without it, the second target
  in the canonical fixture would only ever get a produced-rate verdict,
  never delivered.

  **`sim_export.rs --multi` mode (tracked example, allowed per task
  brief).** `--multi <item>:<rate> [<item>:<rate> ...]` replaces the two
  positionals with N >= 1 targets, consumed greedily until the next
  `--flag`; every existing flag still applies after the target list. The
  example now ALWAYS calls
  `solve_multi_with_palette_exclusions_quality_and_modules` internally
  (single target or not) rather than forking on N — bit-identical at N=1
  by the same construction guarantee, so there is one solve code path in
  the example, not two to keep in sync. Also now prints each
  `external_outputs` entry (`target output <item> <rate>/s`), previously
  only `external_inputs` were printed — a one-line addition that matters
  once there's more than one target to see.

  **Verification.** `cargo test --manifest-path crates/core/Cargo.toml`
  — one clean invocation, 0 failures: lib 927 passed (926 Phase-2
  baseline + 1 new `n1_equivalence_holds_at_solver_rs_multi_wrapper`),
  `solver_multi_target` 8 passed (+1), every other suite unchanged
  (e2e 70/34-ignored, `layout_multi_target` 3, `solver_netflow_parity`
  11/3-ignored — all identical to the Phase 2 baseline). `cargo test -p
  spaghettio_sim_harness` — 65 passed (64 Phase-2-era baseline + 1 new
  `second_target_gets_its_own_delivered_rate_verdict`), 0 failed,
  including 3 new `scenario.rs` tests pinning the `TARGETS`
  array/`checkpoint_items()` Lua emission. `cargo clippy -p
  spaghettio_core -- -D warnings` (exact hook invocation) and `cargo
  clippy -p spaghettio_sim_harness -- -D warnings` both clean.
  `wasm-pack build crates/wasm-bindings --target web --out-dir
  .../web/src/wasm-pkg` — real build, clean, generated
  `solve_multi(targets: Target[], ...): SolverResult` and `export
  interface Target` in the `.d.ts` exactly as designed. `cd web && npx
  tsc --noEmit` clean; `npx vitest run` — 41 passed (state.ts's new
  "multi-target extras" describe block: 5 new tests, all passing).
  **Browser eyeball**: `?item=electronic-circuit&rate=10&craft=
  assembling-machine-2&targets=electronic-circuit:10;advanced-circuit:3`
  on the dev server — one console error (`favicon.ico` 404, unrelated),
  layout renders "159×96 2169 entities" in the console log and "139
  MACHINES" in the sidebar, matching `sim_export --multi`'s own numbers
  on the identical target/rate/tier combination exactly (see the final
  gate below) — the wasm/URL path and the native `sim_export` path agree
  bit-for-bit on machine count and entity count, independent
  confirmation the wasm boundary is wired correctly, not just
  unit-tested.

- *2026-08-01 — Final gate: kill criterion 4 measured FAIL, kill
  criterion 3 measured FAIL (instrument-contaminated, not a proven
  layout defect), an instrument-repair adjudication after two good-faith
  fix attempts within budget.*

  **Fixture.** `sim_export --multi electronic-circuit:10
  advanced-circuit:3 --tier assembling-machine-2` (native mechanism,
  default candidate search — the same shipped-default engine
  `ec_ac_default_options_candidate_choice` pins as choosing `native`):
  159×96, 2169 entities, 139 machines, **0 validation errors, 4
  warnings** (the pre-existing copper-cable `input-rate-delivery`
  residual the Phase 2 decision log already named and left
  uninvestigated — unchanged by this phase). The two solo baselines for
  KC4: `electronic-circuit@10/s` alone (78×36, 671 entities, 57
  machines) and `advanced-circuit@3/s` alone (87×70, 1359 entities, 82
  machines), same tier/inputs, built the same way.

  **Kill criterion 4 — measured FAIL.** The shared build does not beat
  naive concatenation on machines or area:

  | Metric | Combined (EC+AC) | EC solo | AC solo | Naive sum/estimate | Combined vs. naive |
  |---|---:|---:|---:|---:|---:|
  | Machines | 139 | 57 | 82 | **139** (sum) | **0.0% — exact tie** |
  | Total entities | 2169 | 671 | 1359 | 2030 (sum) | **+6.8% — worse** |
  | Bbox area | 159×96 = 15264 | 78×36 = 2808 | 87×70 = 6090 | 8898 (area sum) | **+71.6% — worse** |
  | Bbox area (alt.) | 15264 | — | — | 9222 (vertical-stack estimate: max(78,87)×(36+70)) | **+65.5% — worse** |
  | Bbox area (alt.) | 15264 | — | — | 11550 (horizontal-stack estimate: (78+87)×max(36,70)) | **+32.2% — worse** |

  **Machines tie, they don't lose — but a tie is not a "beat," and KC4's
  bar is "beat," explicitly**: root-caused, not just observed. Per-item
  machine counts are IDENTICAL between the combined solve and naive
  concatenation's sum for every shared item (copper-plate 48=24+24,
  iron-plate 26=16+10, copper-cable 20=10+10, electronic-circuit
  11=7+4, plus advanced-circuit 24, basic-oil-processing 7, plastic-bar
  3 unchanged either way). This is not a coincidence of THIS fixture
  under-exercising the dedup mechanism — it is a mathematical property
  of ceiling: `ceil(a) + ceil(b) >= ceil(a+b)` always, with EQUALITY
  whenever `frac(a) + frac(b) <= 1`. The RFC's own solver-level probe
  (Motivation table, ~10% overhead) compared RAW pre-ceiling LP totals
  (124.3 estimated combined vs. 137.9 naive); the REAL Phase 1 numbers,
  once every row is ceiled to a buildable integer machine count (what a
  physical factory actually needs), show that for the EC@10/AC@3 rate
  pair specifically, no shared item's two fractional roundings
  compound past an extra whole machine — the dedup mechanism is real
  (`kc2_ec_ac_shared_copper_cable_exact` proves the LP never re-derives
  the hand-sum shortcut's wrong numbers) but pays a machine-count
  dividend of exactly zero at these particular rates. A different rate
  pair, chosen so shared items' fractional parts sum past 1, would show
  a real (if likely small) machine saving — this fixture's numbers are
  not evidence the mechanism can never win, only that it didn't here.

  **The shared bus's own routing overhead is the real cost, and it's not
  small.** The extra ~139 entities and ~67-72% larger bbox come from the
  machinery the dual-purpose lane needs that two independent bus builds
  don't: the west-flowing return-tap belt sharing the EC row's own
  output tile, the trunk continuing south past the AC tap-off to the
  perimeter exit, the associated junction/crossing-solver work at the
  collision seed Phase 0 first diagnosed, and the second physically
  separate export path itself. **Verdict: KC4 FAILS as measured on the
  canonical fixture.** The correctness case for RFC-062 stands on its
  own (KC1, KC2, KC5 below), but the economic case — "beats the always-
  available naive-concatenation alternative" — does not hold at these
  rates. Per the RFC's own kill-criterion text: "stop and either find
  the win or recommend concatenation as the shipped answer." No further
  tuning was attempted (budget; the RFC's own review-fix precedent name
  this class of finding as something to record, not chase to zero this
  session) — recommending naive concatenation as the correct choice for
  a caller who has already priced dedup against routing overhead is
  the honest reading of this measurement, not a defect to patch.

  **Kill criterion 3 — measured FAIL, with the failure isolated to a
  newly-found sim-harness instrument gap, not proven to be a layout
  defect.** One headless run, `--warmup 288000`, converged (`final_tick:
  297060`):

  | Item | Planned | Measured produced | Δ% | Measured delivered | Δ% | Verdict |
  |---|---:|---:|---:|---:|---:|---|
  | advanced-circuit (2nd target) | 3.00/s | 2.98/s | −0.7% | 3.04/s | +1.3% | **PASS** |
  | electronic-circuit (1st target) | 16.00/s* | 6.04/s | −62.3% | 0.00/s | −100.0% | **FAIL** |
  | copper-cable (intermediate) | 60.00/s | 30.20/s | −49.7% | — | — | (context only) |
  | copper-plate (intermediate) | 30.00/s | 15.00/s | −50.0% | — | — | (context only) |
  | iron-plate (intermediate) | 16.00/s | 6.08/s | −62.0% | — | — | (context only) |
  | petroleum-gas (intermediate) | 60.00/s | 55.13/s | −8.1% | — | — | (context only) |
  | plastic-bar (intermediate) | 6.00/s | 5.95/s | −0.8% | — | — | (context only) |

  \* `electronic-circuit`'s planned rate is the row's COMBINED demand
  (10/s own target + 6/s AC's ingredient draw), not the 10/s the user
  actually requested — `planned_rates` is keyed by total row output,
  matching every other item in the table; the per-item report doesn't
  currently split "target-only" from "combined-row" planned rates for a
  dual-purpose lane, a minor reporting gap worth a follow-up but not the
  cause of this failure (delivered=0.00 fails against ANY nonzero
  denominator).

  **advanced-circuit is genuinely clean** — PASS on both produced and
  delivered, `kit_errors: []`, `fluid_errors: {}`, `converged: true`.
  This proves the shared-row mechanism CAN deliver correctly end to end
  when its physical export path works: the second target in a
  multi-target factory is not inherently harder to measure or build,
  Phase 2's F1 fix (the off-by-one that shifted `belt-flow-reachability`
  boundaries) holds, and this phase's own per-item checkpoint series
  (`checkpoint_items()`) does exactly what it was built for — AC's PASS
  verdict is computed from ITS OWN delivered-rate series, not a
  produced-only fallback, the concrete proof KC3's harness prerequisite
  (Phase 3's per-item tracking) actually works.

  **electronic-circuit's 0.00 delivered rate is a real, reproducible,
  root-cause-investigated instrument gap — characterized per
  `docs/sim-harness-forensics.md`'s playbook, not accepted at face
  value.** Frame-reading (`sim_state`, forensics playbook step 2) on the
  first (pre-fix) run found the actual defect: `electronic-circuit`'s
  export physically routes through the dual-purpose lane's
  `surplus_exits` claim (Phase 2's mechanism — a solid TARGET can now
  legitimately exit there, not just a byproduct), and the sim harness's
  manifest-to-kit builder treated every `surplus_exits` entry as a
  voidable fluid byproduct unconditionally (`add_fluid_void`, a
  Factorio infinity-pipe filter — meaningless for a solid item name):
  `fluid_errors: {"electronic-circuit@void": "Unknown fluid name:
  electronic-circuit"}`, no drain rig ever built, delivered pinned at a
  flat 0.00 by construction, not by measurement.

  **Fix 1 (landed, confirmed effective): reclassify at the manifest
  layer.** `blueprint.rs::export_with_manifest` now promotes a
  `surplus_exits` entry into a real `boundary_outputs` `BoundaryRecord`
  (direction looked up from the actual belt entity at that tile, the
  same entity-cross-check standard `check_stranded_byproducts` already
  holds surplus exits to) whenever its item is a solid member of
  `solver.external_outputs` — a target, not a byproduct. Unit-tested
  directly (`solid_target_surplus_exit_is_promoted_to_boundary_output`)
  and confirmed live: this run's `fluid_errors: {}` has NO
  `electronic-circuit@void` entry — the original defect is gone. The
  manifest reclassification is correct and doing its job.

  **Residual (NOT fixed within budget, two good-faith attempts made and
  both falsified): the promoted rig's own belt extension silently fails
  to place.** `sim_state` on this run shows electronic-circuit's drain
  rig built EXACTLY per `add_drain`'s formula for its chest/inserter
  bank (18 chests, all empty; 18 stack-inserters, all
  `waiting_for_source_items`, at the exact tiles `ext_len=13` predicts)
  — but ZERO belt entities exist anywhere in the 13-tile extension
  footprint the SAME function is supposed to build first. This is not a
  hypothesis; it is a direct entity-census finding, cross-checked
  against `advanced-circuit`'s own (working) drain rig at the same
  y-range for contrast.

  - *Attempt 1: surface the failure loudly.* `add_drain`'s belt-creation
    loop had no error handling at all (unlike `add_fluid_void`/
    `add_fluid_feed`, which both `pcall` and record a failure) — a
    silent `create_entity` failure was structurally invisible. Added a
    `kit_errors` entry on any placement failure. Landed
    (`drain_belt_placement_failure_is_recorded_not_silent`), but on
    this run **it did not fire** (`kit_errors: []`) — meaning
    `create_entity` is not failing in the specific "returns nil" shape
    the fix instruments, or the actual failure point is somewhere this
    fix doesn't reach. A genuine, informative negative result, not a
    wasted change: it narrows the search space for whoever picks this
    up next.
  - *Attempt 2: chunk-generation truncation.* This codebase has an
    EXACT documented precedent for this failure signature — the #345/
    PU@4 forensics note (`gen_radius` comment, `orchestrate.rs`/
    `scenario.rs`): entities placed via `create_entity` on an
    ungenerated chunk are silently dropped, previously seen as "dead
    feed rigs, NO DATA" on an oversized fixture. Added an explicit
    per-rig `request_to_generate_chunks` + `force_generate_chunk_requests`
    call in `add_drain`, sized around the rig's own far end, independent
    of the global origin-centered radius. Landed
    (`drain_rig_explicitly_generates_its_own_chunk_footprint`), but the
    live re-run (this one) is **bit-for-bit identical** to the pre-fix
    run on every measured number (produced 6.04, delivered 0.00, census
    57/1/81) — given Factorio's headless scripted runs are deterministic
    for identical inputs, an identical result across two runs with
    different `add_drain` Lua bodies is direct evidence this specific
    fix changed nothing observable in the simulated world. The theory
    is further undermined by the chest/inserter evidence itself: those
    entities place successfully at the SAME y-range (96–108) the belt
    extension spans — if chunk generation were the limiter, the chests
    should fail too, and they don't. **Chunk-generation truncation is
    ruled out**, not confirmed.

  **Root cause: not identified within this session's budget.** Two
  plausible, low-risk, well-precedented hypotheses were tested and both
  falsified by direct measurement, not assumption — this is the honest
  state of the investigation, not a stopping point chosen for
  convenience. `advanced-circuit`'s own drain rig (same mechanism, one
  `boundary_outputs` index earlier, `ext_len=11` vs `ext_len=13`, at
  world x≈10 vs EC's world x≈-78) works; EC's does not. What's confirmed
  ruled OUT: chunk generation, the manifest's direction/entity
  classification (fixed and verified), Lua syntax errors (the run
  converges cleanly, no engine-level errors in the Factorio log). What's
  NOT yet tested: whether `idx`-dependent `ext_len` growth itself, or
  the rig's specific WORLD position (large negative x, near the
  origin-centered paste's own edge), is the actual discriminator — a
  live, reproducible repro now exists for whoever picks this up (this
  exact fixture, `sim_export --multi electronic-circuit:10
  advanced-circuit:3 --tier assembling-machine-2`, then `spaghettio-sim
  run --warmup 288000`) — tracked as a followup, below.

  **Verdict, precisely stated**: KC3's own bar ("zero validation errors
  AND sim-measured at plan on BOTH targets") is **not met** — this is a
  real FAIL, not a technicality. But the failure is **instrument-
  contaminated, not proof the layout under-delivers on its actual
  merits**: `advanced-circuit`'s clean PASS demonstrates the native
  shared-row mechanism CAN measure correctly; `electronic-circuit`'s own
  produced-rate shortfall (6.04/s vs. its 16/s combined demand, settling
  suspiciously close to exactly AC's own 6/s internal draw) is the
  signature of a factory correctly throttling to what its ONLY working
  outlet (AC's ingredient consumption) can absorb once its export sink
  genuinely has nowhere to go — a real in-game consequence of the
  harness's own gap, not independent evidence the layout's export
  capacity is undersized. Per the task brief's own instruction ("if KC3
  fails: that is a REAL RESULT — do not tune until green"): this is
  recorded as a FAIL, with the instrument-contamination explicitly
  named rather than asserted away, and no further sim-harness attempts
  were made after the second falsification (budget).

  **Verification.** `cargo test --manifest-path crates/core/Cargo.toml`
  — one clean invocation post-merge, 0 failures: lib 928 passed (927 +
  1 new `solid_target_surplus_exit_is_promoted_to_boundary_output`),
  every other suite unchanged from the prior entry. `cargo test -p
  spaghettio_sim_harness` — 67 passed (65 + 2 new: the kit_errors
  reporting test and the chunk-footprint test), 0 failed. `cargo clippy
  -p spaghettio_core -- -D warnings` and `cargo clippy -p
  spaghettio_sim_harness -- -D warnings` both clean. Two live headless
  sim runs total across this final-gate investigation (the budgeted one
  plus one instrument-artifact-suspected retry, per the task brief's
  explicit allowance — both runs' `bp.txt`/`manifest-real.json`
  regenerated fresh from the fixed `blueprint.rs` before each launch).

- *2026-08-01 — RFC close-out: all five phases adjudicated. PARTIAL —
  engine correctness lands and holds; the two measurement-bar kill
  criteria (KC3, KC4) that were meant to prove the feature pays for
  itself did not clear on the canonical fixture.*

  **Phase-by-phase.**
  - **Phase 0** (layout spike): kill criterion 1 PROCEED — the
    shared-row collision is fixable by generalizing the dual-purpose
    lane, confirmed with a wider fix shape than the leading hypothesis.
  - **Phase 1** (solver): kill criteria 2 and 5 both PASS, exact —
    `solve_netflow_multi` reproduces naive concatenation's per-item
    machine totals precisely (copper-cable 20.0, not the hand-sum
    shortcut's 16.0) and is bit-identical to every scalar entry point at
    N=1 by construction, not just by test.
  - **Phase 2** (layout): kill criterion 1 re-cleared on the real
    Phase-1 solver output (not just Phase 0's hand-built probe); the new
    outflow-conservation validator invariant shipped two-sided
    (over-claim AND under-claim), with a severity split for legitimate
    ceil-slack after adversarial review. One pre-existing bug found and
    deliberately NOT fixed in-phase (D2b uranium, below).
  - **Phase 3** (this work): the wasm `solve_multi` entry, boundary
    validation, minimal URL extension, and harness per-item checkpoint
    series all landed and are independently verified working — the
    browser eyeball and `sim_export --multi` agree bit-for-bit on
    machine/entity counts, and `advanced-circuit`'s clean sim PASS
    proves the per-item harness plumbing does what it was built for.
  - **Final gate** (kill criteria 3, 4): **both FAIL as measured** on
    the canonical `electronic-circuit@10/s` + `advanced-circuit@3/s`
    fixture — see the entry directly above for the full evidence. KC4
    ties on machines (a real, root-caused ceiling-arithmetic property of
    THESE rates, not a bug) and loses on area/entity count (real routing
    overhead, not measurement noise). KC3 fails on `electronic-circuit`
    specifically, isolated via forensics-playbook investigation to a
    newly-found, not-yet-root-caused sim-harness instrument gap — NOT
    evidence the layout itself under-delivers (`advanced-circuit`'s own
    clean PASS on the identical fixture is the control that makes that
    reading defensible, not just hoped-for).

  **Why PARTIAL, not CONCLUDED/FAILED**: the RFC's motivating
  correctness argument — naive concatenation risks the hand-sum
  shortcut's silent under-count, and a real combined solve closes that
  gap while deduping shared upstream — is proven true and shipped
  (Phases 1–2, holding through Phase 3's own regression suite). What did
  NOT clear is the RFC's economic promise that the combined build is
  worth its added complexity over always-available naive concatenation.
  Per the RFC's own kill criterion 4 text ("stop and either find the win
  or recommend concatenation as the shipped answer") and the task
  brief's explicit "characterize honestly, don't tune until green"
  instruction: **the honest recommendation for a caller choosing today,
  at rates resembling this fixture, is naive concatenation** — solve and
  lay out each target independently. The multi-target solver/layout
  machinery ships (it is correct, tested, and the only way to avoid the
  hand-sum bug if a caller ever needs it), but its economic case is
  unproven at the one rate pair actually measured, and its
  sim-verifiability is blocked on an unresolved harness gap for exactly
  the row shape (solid target routed through the dual-purpose lane) that
  makes the layout side of this RFC interesting in the first place.

  **Deferred items, enumerated** (per the task brief's explicit ask —
  everything below is a real, named gap, not swept into "future work"
  silently):
  1. **D2b uranium bug** (Phase 2 review, `u235_u238_target_and_surplus_overlap`):
     `ghost_router.rs` Step 7's per-item output merge sources every
     item's belt-y from the row's PRIMARY `output_belt_y`, with no
     branch for `RowSpan::secondary_output_belt` — when BOTH of a D2b
     row's distinct solid outputs (uranium-235 primary, uranium-238
     secondary) become external targets simultaneously (only reachable
     once multi-target solving exists), uranium-238 silently merges from
     uranium-235's belt: 12 real validation errors (8 entity-overlap, 4
     belt-item-isolation) on the adversarial fixture. Fix shape
     identified (Step 7 should resolve per-row belt-y via
     `RowSpan::output_belt_y_for(item)`, the same helper `lane_planner`
     already uses for exactly this purpose) but not applied — a change
     to different, older machinery than this phase's scope, deserving
     its own investigation.
  2. **The target+taps+surplus three-way-collision fixture** (Phase 2
     review): target + internal taps + surplus ALL live on one item at
     once (e.g. U-238 as a target, internally consumed, AND carrying
     surplus) — a genuinely untested shape distinct from both of this
     RFC's own adversarial fixtures (EC+AC: target+taps, no surplus;
     U235+U238: target+surplus, no taps). No recipe in the current DB
     naturally produces this shape without a synthetic `SolverResult`;
     building a trustworthy one is real, scoped work, not a fix-round
     add-on.
  3. **Full multi-target sidebar UI**: explicitly out of scope per the
     RFC's own Interface section from the start — the editing UI stays
     single-target-only; a multi-target URL renders (verified, browser
     eyeball) but there is no way to BUILD one through the UI. Own
     follow-up once/if the economic case (KC4) is revisited and cleared
     at some other rate pair.
  4. **The KC3 sim-harness instrument gap** (new this phase, the highest-
     priority of the four): `add_drain`'s belt-extension placement fails
     silently for a promoted (non-native) `boundary_outputs` record at
     `boundary_outputs` index >= 1 — root cause not identified after two
     falsified hypotheses (chunk-generation truncation, ruled out by the
     chest/inserter counter-evidence; a silent `create_entity` failure
     the new error-reporting doesn't catch). A live, reproducible repro
     exists (this exact fixture + command, see the final-gate entry).
     Blocks sim-verifying ANY future multi-target fixture where a
     non-first target's export routes through the dual-purpose lane —
     which is exactly the shape this RFC's own layout work was built
     to enable.
  5. **The copper-cable `input-rate-delivery` residual** on the EC+AC
     fixture (Phase 2, never investigated further): 4 warnings,
     unchanged through Phase 3 — an ordinary multi-consumer lane
     under-delivery, not itself a target so `solid_target_items` never
     gates it. Worth a look independent of this RFC's own close-out.

  **Verification**: see the final-gate entry above and each phase's own
  entry for the specific commands run. No golden/snapshot files touched
  by Phase 3 or the final gate; the full `cargo test --manifest-path
  crates/core/Cargo.toml` suite and `cargo test -p
  spaghettio_sim_harness` both green at every commit in this phase.
