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
2. **Smoke-test the bake.** Re-run `SPAGHETTIO_BAKE_BATCH=1
   SPAGHETTIO_PURE_ROUTING_ENCODING=circuit cargo run --release -p
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

- *2026-05-02 — Phase 1 implemented. Module landed at
  `crates/core/src/bus/template_validate.rs` (not in `balancer-gen`,
  which is a bin crate). Audit test at
  `crates/core/tests/balancer_lane_audit.rs`, run with
  `cargo test --manifest-path crates/core/Cargo.toml --test
  balancer_lane_audit -- --nocapture`. Synthesis is small (~30 LOC of
  plumbing — well under the 50 LOC kill budget): convert template
  entities to `PlacedEntity`s, mark only the input tiles as carrying a
  sentinel item, and feed a `SolverResult` whose only external input is
  that item at `belt_throughput * min(N, M)`. All four lane-relevant
  validators run against the synthesized `LayoutResult`.*

  **Headline numbers (75 templates audited):**

  | metric | count | % |
  |--------|------:|---|
  | templates with errors | 50 | 66.7% |
  | templates with warnings | 22 | 29.3% |
  | total error issues | 373 | — |
  | total warning issues | 53 | — |

  **Errors by category:**

  | category | total occurrences | templates affected |
  |----------|------------------:|-------------------:|
  | lane-throughput | 348 | 48 |
  | underground-belt (unpaired) | 25 | 6 |

  **Warnings by category:**

  | category | total occurrences |
  |----------|------------------:|
  | underground-belt (sideload-into-input) | 53 |

  **Top 10 worst-offender templates:**

  | rank | (m, n) | errors | warnings |
  |-----:|--------|-------:|---------:|
  | 1 | (7, 4) | 27 | 0 |
  | 2 | (9, 7) | 26 | 2 |
  | 3 | (9, 5) | 24 | 1 |
  | 4 | (2, 9) | 22 | 10 |
  | 5 | (7, 5) | 22 | 0 |
  | 6 | (9, 8) | 22 | 2 |
  | 7 | (1, 9) | 16 | 2 |
  | 8 | (5, 8) | 16 | 0 |
  | 9 | (9, 6) | 16 | 1 |
  | 10 | (6, 3) | 11 | 0 |

  **Triage — kill criterion fired (>30% error rate).** Per the RFP, this
  means stop and triage before declaring Phase 1 done. Triage outcome:

  1. **Lane-throughput (348 / 48 templates) — DOMINANTLY VALIDATOR-SIDE
     ARTIFACT, not real lane bugs.** Spot-checking the simplest cases:

     - (1, 3) reports 15.0/s on both lanes at internal tiles (1, 2) and
       (1, 3). Input is only 15/s total, so 15/s/lane = 30/s on a single
       belt is non-physical (4× the input). The (1, 3) template uses a
       feedback-loop column at `x=0` for recirculation; the lane
       walker's cycle-breaker fallback (`belt_flow.rs:2187-2335`)
       propagates the non-zero side of a splitter pair to *both* tiles
       when one side is in a [0,0] feedback cycle. That's correct for
       avoiding spurious starvation, but it produces non-physical
       inflated rates when the cycle-breaker fires multiple times in
       sequence.
     - (3, 2) reports 10/s on both lanes at the internal tile (1, 9).
       Input rate per source = 30/3 = 10/s; the rate showing up at an
       internal tile suggests the same cycle-breaker doubling pattern.

     The lane walker is documented as Kahn-topo-sort with cycle-breaker
     fallback, designed for the bus pipeline (where templates don't
     stand alone — they sit between trunks and consumer rows). Standing
     alone, a template with internal recirculation is a hard case the
     walker is not validated against. **The lane-throughput findings
     are unreliable in this audit context.**

  2. **Underground-belt unpaired errors (25 / 6 templates) —
     ACTIONABLE.** All 6 affected templates are in the recently-baked
     (9, *) family: (9, 1), (9, 2), (9, 7), (9, 8) plus the (1, 9) and
     (2, 9) and (1, 10). Sample:

     ```
     (9, 7): Unpaired underground belt input at (2, 9) facing East: no matching output found
     (9, 7): Unpaired underground belt input at (8, 9) facing West: no matching output found
     (9, 7): Unpaired underground belt output at (3, 9) facing East: no matching input found
     (9, 7): Unpaired underground belt output at (7, 9) facing West: no matching input found
     ```

     Pattern: the templates have East-facing UGs at column 2 paired
     with East-facing outputs at column 3 (one cell apart, distance 1),
     and similarly West-facing UGs paired one cell apart. The
     `check_underground_belt_pairs` validator rejects pairs at
     `dist <= 1` because a UG with no underground tile between input
     and output is degenerate.

     **This is a real bug in the (9, *) bake outputs.** Either the
     CP-SAT encoding is allowing degenerate UG pairs (1-tile reach) or
     the entity emission is dropping the intermediate tile. Should be
     filed as a separate issue against `balancer-gen` and fixed before
     these templates ship via the bake.

     A separate pattern affects (1, 10): `check_underground_belt_pairs`
     reports unpaired UGs there too — same diagnostic path.

  3. **Underground-belt sideload-into-input warnings (53 across 22
     templates) — ACCEPTED FINDING, MX5 LANE CAVEAT.** Every `(N, *)`
     template for N >= 6 has at least one UG input fed by a
     perpendicular belt. Memory `feedback_sideload_ug.md` says this
     fills only the far lane — a real lane-imbalance, but consistent
     with the throughput-priority audit's existing decision to accept
     "side-loaded splitter inputs as MX3 at topology level despite the
     lane caveat" (RFP throughput-priority decision log 2026-05-01).

     These warnings are exactly what the RFP predicted: "every new
     composed template is potentially broken in-game and we won't know
     until users hit it." But: the existing library entries are
     battle-tested by the Factorio-SAT community, so the warnings
     largely reflect a known caveat, not a new bug. The new (9, *)
     bakes inherit the same pattern.

  **Recommendation: PHASE 2 IS WORTHWHILE, but with a narrower gate
  than originally drafted.** Specifically:

  - **Gate on `underground-belt` ERRORS only** (the unpaired-UG check).
    These are cheap, deterministic, and catch real degenerate
    encodings. Reject + bump on these.
  - **Do NOT gate on `lane-throughput`** — the walker's cycle-breaker
    behavior on standalone templates is too noisy. If we want
    throughput-level lane validation in the bake, we need to either
    (a) wrap the template in a "saturated source above, consumer
    below" sandwich large enough to break the recirculation cycles
    cleanly, or (b) write a template-specific lane walker that
    understands balancer feedback loops.
  - **Do NOT gate on UG-sideload WARNINGS** for now — these match
    existing accepted practice. Track separately if/when MX5 becomes a
    project-level priority.
  - **File a separate bug** against `balancer-gen` for the (9, *) and
    (1, 10) unpaired-UG outputs. Fix before merge of the bake's most
    recent additions.

  **Validator caveats discovered during implementation:**

  - Setting `carries` on every entity (initially) caused splitter tiles
    with no upstream feeders (common: first splitter in a column) to be
    classified as "external sources" in `compute_lane_rates`'s
    seed-rate pass, double-injecting the external rate. Fixed by
    restricting the `carries` tag to the explicit `input_tiles`.
  - The lane walker's saturated-source seeding distributes rate evenly
    across source tiles. For asymmetric mergers (N > M), this works
    because we cap external rate at `belt_throughput * min(N, M)` so no
    side is over-driven. But for templates that internally recirculate
    flow across "back-loop" belts (e.g. universal-balancer patterns),
    the cycle-breaker is the source of the inflated lane-throughput
    findings — see point (1) above.

  **Phase 1 deliverable status:** module + audit test land in this
  worktree. Findings recorded above. Phase 2 remains worth doing per
  the recommendation, with the narrowed scope.*
