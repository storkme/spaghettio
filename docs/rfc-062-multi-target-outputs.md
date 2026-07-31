# RFC-062: Multi-target output support

**Status**: Design
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
