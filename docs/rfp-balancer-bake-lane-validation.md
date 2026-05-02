# RFP: Lane-aware validation of baked balancer templates

## Summary

`bake_missing_shapes` currently accepts any composed template that
`classify_ref` calls `Balanced` (MX3 at the splitter-graph level). That
check models splitters as default 50/50 distributors over single-lane
flow — it cannot detect lane-level imbalance (the MX5 caveat in
[`docs/rfp-throughput-priority-merges.md`](rfp-throughput-priority-merges.md)).
A composed template can pass `classify_ref` while sideloading onto a UG
input (filling only the far lane) or re-pairing UGs in a way that
quietly halves throughput on one lane. As we scale to tier-5/6 shapes,
this risk compounds — every new composed template is potentially
broken in-game and we won't know until users hit it.

Add a lane-aware validation step in the bake pipeline: stamp the
composed template into a minimal `LayoutResult`, run the existing
lane-aware validators in `crates/core/src/validate/`, and reject (with
a bumped `jh`) on any lane-imbalance / sideload-violation finding.

## Motivation

### Concrete failure mode (predicted, not yet observed)

The repo already documents the lane-rule trap in two places:

- Memory `feedback_sideload_ug.md`: *"Must feed UG inputs straight, not
  from the side, to load both lanes."*
- Memory `feedback_belt_lane_rules.md`: *"UG sideload fills one lane;
  UG exit blocks sideload from near side; MUST validate."*

The phase-1 audit in `rfp-throughput-priority-merges.md` (decision log
2026-05-01) accepted side-loaded splitter inputs in Factorio-SAT
templates as "MX3 at topology level despite lane caveat" because
in-game testing was deferred. That was acceptable for the *existing*
library (those templates are battle-tested). It is **not** acceptable
for the templates `bake_missing_shapes` is now generating fresh — those
have never run in Factorio.

The (4, 9) Clos compose tonight produced 8 UGs in a 12×30 layout. We
classified Balanced and shipped (well, would have shipped after merge).
None of the 8 UGs has been checked against the entry-sideload or
exit-block rules.

### Why right now

- We just unblocked the bake (circuit fix). The next 7 (n, 9) shapes
  will go through the bake overnight.
- Tier-5 (processing-unit) needs shapes outside the current envelope.
  Each new shape is a fresh untested composition.
- A "ship MX3 templates that are lane-broken" failure mode is the
  worst kind: silent in tests, only visible under saturated load, and
  the diagnosis path is "users complain, we trace lanes by hand."

## Design

### New module: `crates/balancer-gen/src/template_validate.rs`

Public API:

```rust
pub fn validate_template_lanes(
    composed: &OwnedTemplate,
) -> Result<(), Vec<ValidationIssue>>;
```

Returns `Ok(())` if all lane-aware validators pass; `Err(issues)`
otherwise. The bake pipeline rejects the candidate `jh` and bumps if
this returns `Err`.

### Algorithm

Step A — synthesize a `LayoutResult`. The validators in
`crates/core/src/validate/belt_flow.rs` operate on `&LayoutResult`
(see `check_lane_throughput` at line 1797, `compute_lane_rates` at
1790, `check_underground_belt_sideloading` at 1405,
`check_underground_belt_entry_sideload` at 1449). Build a minimal
`LayoutResult` from the `OwnedTemplate`:

- `entities`: all template entities, copied verbatim.
- `connections`: empty (validators that need this fail safely or are
  skipped — see Step C).
- `dimensions`: `(template.width, template.height)`.
- Optionally append straight-feed input belts above the input row and
  output belts below the output row, so input and output ports have
  upstream/downstream context the validators expect. (One row of
  saturated source belts above; one row of consumer belts below.)

Step B — run the lane-relevant subset of validators:

| Validator | What it catches |
|-----------|-----------------|
| `check_lane_throughput` | MX5 — outputs deliver less than expected per-lane |
| `check_underground_belt_sideloading` | UG sideload rule violations (B8/U7) |
| `check_underground_belt_entry_sideload` | UG-input sideload only fills far lane |
| `check_underground_belt_pairs` | Re-pairing collision (we already model this in CP-SAT, so this should pass; canary if it doesn't) |

Step C — reject + bump policy:

- Any `Severity::Error` issue → reject this `(jh, candidate layout)`,
  bump `jh` and re-run `compose_series`.
- `Severity::Warning` issues → log but accept (tier-4 already ships
  with warnings on some shapes).

### Integration into `bake_missing_shapes`

In `crates/balancer-gen/src/main.rs:1416`, after `classify_ref` returns
`Balanced`, add:

```rust
if let Err(issues) = validate_template_lanes(&composed) {
    println!("  ✗ lane-validate: {} errors, bumping jh", issues.len());
    // Re-run compose_series with min_jh = current_jh + 1
    composed = match retry_with_higher_jh(...) {
        Ok(c) => c,
        Err(_) => { shapes_failed.push(...); continue; }
    };
}
```

The retry budget is bounded — if `max_jh` is exhausted the shape is
marked failed (same as a compose-side INFEASIBLE).

### Trade-offs considered

- **Trust `classify_ref` and ship without lane-validation.** Status
  quo. Rejected: stated reason above (silent shipping failures).
- **Run the *full* validator suite (all 23 checks).** Rejected: many
  are concerned with bus-context things (input rate delivery, power
  coverage) that aren't applicable to a standalone template. Pick the
  lane-relevant subset deliberately.
- **In-game spot-checks only, no automated lane validator.**
  Rejected: doesn't scale to overnight bakes of dozens of shapes;
  manual checking is the 2026 equivalent of "test in production."
- **Build a from-scratch lane simulator.** Rejected: the existing
  validators in `belt_flow.rs` are battle-tested on the bus
  pipeline. Reusing them is cheap; a new simulator is a new bug
  surface.

## Kill criteria

- **Validators don't accept a `LayoutResult` synthesized from a
  template.** If the validators have hard dependencies on
  `LayoutResult` fields we can't synthesize (e.g. `connections` with
  specific shape expectations), and the synthesis grows beyond
  ~50 LOC, abandon and write a template-specific lane walker
  instead.

- **No existing template in the library fails the new validator
  (i.e. the validator finds zero issues across all 75 library
  templates AND zero issues across the 7 newly-baked shapes).** Then
  this RFP delivers no actual safety improvement — every template is
  already lane-correct by construction. Document the finding and
  remove the validation step from the bake (it's just runtime cost).
  This is a "good news, drop the work" outcome.

- **>30% of currently-shipping library templates fail the new
  validator.** Then either (a) the validator has bugs, or (b) the
  library has been quietly shipping lane-broken templates and we have
  a much bigger problem. Either way, stop and triage before letting
  it gate the bake — pulling 30% of the library would block all
  recipes.

- **Adding lane-validation makes the bake >2× slower per shape.**
  Then the validator implementation is too heavy and we drop it
  unless we can profile and fix.

## Verification plan

Per [the layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Existing library is the ground truth corpus.** Run the new
   validator across all 75 current templates; expect zero or near-zero
   errors. Document any unexpected failures as either validator bugs
   or library bugs (the audit hat from #266).
2. **Smoke-test the bake.** Re-run `FUCKTORIO_BAKE_BATCH=1
   FUCKTORIO_PURE_ROUTING_ENCODING=circuit cargo run --release -p
   balancer-gen` after the change. The 11 already-baked shapes should
   re-validate cleanly. The 7 new (n, 9) shapes should produce a
   lane-validate result per shape, and any rejections should bump
   `jh` and re-converge.
3. **Hand-verify a known-safe template.** `(4, 4)` Benes — should pass
   all lane checks trivially (no UGs, all symmetric).
4. **Hand-verify a known-tricky template.** Any template using
   sideloaded splitter inputs (the audit found 15 of these). These
   should pass the validator if Factorio-SAT got the lane semantics
   right. Failures here are findings worth filing.
5. **Trace events.** No new trace events; this is offline validation.
6. **Clippy clean.**

## Phasing

- **Phase 1 — implement validator + run on existing library.** No
  bake-pipeline integration yet; produces a one-shot taxonomy table
  of lane-correctness. Deliverable: a printed report similar to the
  classify_ref audit, telling us whether the library has latent
  lane-bugs. Lands as a `#[ignore]`'d test in
  `crates/core/tests/balancer_lane_audit.rs`.
- **Phase 2 — wire into `bake_missing_shapes`.** Reject + bump on
  errors, log warnings.
- **Phase 3 (deferred) — extend to the full bus layout pipeline.**
  Catch lane issues in user-facing flows, not just at bake time.
  Probably deferred until tier-5 actually ships and we know what
  shapes are in flight.

## Decision log

- *2026-05-02 — drafted. Awaiting approval. Can run in parallel with
  spatial-pruning RFP (different files: Rust orchestration here,
  Python encoding there).*
