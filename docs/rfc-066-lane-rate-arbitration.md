# RFC-066: Arbitrate the two lane-rate walkers against the meter

**Status: proposed 2026-08-09.** Tracking issue: #609. No code written.

## Summary

The repo carries two independent Python→Rust ports of the same belt-graph
lane-rate walk, `belt_flow::compute_lane_rates_impl` and
`belt_structural::compute_lane_rates`. They disagree on **58% of tile slots**, and
`validate/mod.rs:1079-1080` dispatches two adjacent checks that each believe a
different one. The dispatched `lane-throughput` — `Severity::Error`, and
selection-participating — reads ~0.0/s on 112,407 tiles where the other model sees
real flow, because its walker never seeds external inputs. It fires on 5 of 504
layouts; the other model would fire on 176.

This RFC proposes to **arbitrate the two against `crates/meter/` as the oracle,
fix the winner, harvest the loser's one correct part, and delete the loser** —
leaving one lane-rate model in the engine. The meter is the thing that makes this
affordable: it is an independent item-level simulator whose KC4 boundary
*forbids* it importing the engine's rate model, so its verdict is not circular.

What you would be agreeing to: a staged, selection-affecting change to an
Error-severity check, gated behind shadow-mode divergence logging and a sim anchor
before it is allowed to steer candidate selection.

## Motivation

Reproducible today, with numbers rather than reasoning. Across 504 layouts
(13 recipes × rates × machine tiers × belt tiers × DI policies):

| | |
|---|---|
| tile slots where the two walkers disagree >0.01/s | **134,702 / 233,441 (58%)** |
| tiles where the DISPATCHED model reads ~0.0/s and the other sees flow | **112,407** |
| — by segment (all, sums to the total) | 57,649 `row:`, 28,859 `trunk:`, 10,588 `ghost:`, 5,661 `di-row:`, 3,959 `crossing:`, 2,452 `feed:`, 1,981 `balancer:`, 872 `corr:`, 327 untagged, 59 `tapoff:` |
| `lane-throughput` firings, dispatched model | **5 of 504 layouts** |
| same check under the other model | **176 of 504 layouts** |

**Why the dispatched model reads zero.** `belt_flow::compute_lane_rates_impl`
seeds graph-source belts carrying external inputs (`belt_flow.rs:2623-2688`), and
says why in its own comment:

> *"without this seeding, rate propagation starts at 0 and every downstream
> consumer of an external input is incorrectly flagged as starved."*

`belt_structural::compute_lane_rates` has exactly one seeding site —
`belt_structural.rs:1033`, *"Seed lane injection rates from output inserters"* —
and no external-input onramp. An unseeded graph *source* propagates zeros to
everything downstream of it, which is every segment category on the input side.

**This is a named suspect for a measured deficit, not a new discovery.**
Both halves are recorded in committed docs:

- `docs/status.md` (stress-EC entry): the two stress fixtures re-measured
  post-lift land at **92.1%** and **90.7%** delivered, and *"a real ~8-10%
  residual remains on both"*. `tier2_electronic_circuit` went 58% → 91% with a
  residual that is *uniform* across both stages (copper-cable 90.0%, EC 90.9%) —
  by the `sim-harness-forensics.md` reading, a SHARED constraint rather than one
  stage bottlenecking.
- `docs/rate-stamp-semantics.md` §"the disagreement is unresolved and it matters"
  independently identifies the same two-walker split, notes the dispatched model
  *"reports 0 tiles over capacity in every arm"* where `belt_flow` flags the S=1
  arms, and concludes that **arbitrating the two models** is the follow-up.
  (That doc cites the dispatch at `validate/mod.rs:939`; at this RFC's base the
  line is 1079 — the code moved, the claim did not.)

So: a measured, unexplained ~8–10% residual, with a named suspect that no one has
arbitrated. A local, uncommitted session handoff
(`handoff-meter-as-gate-2026-08-07.md`) also covers this ground and is where I
first read it, but it is **not in the repo** — nothing in this RFC depends on it,
and the two committed sources above carry the argument.

## Design

### The oracle, and why it is not circular

`crates/meter/` is a native item-level discrete simulator with real per-lane state
(`Lane`, `BeltTile::occupancy`, `near_lane_from`, tests citing B2/B3/B5). Its KC4
integrity boundary (`crates/meter/tests/kc4_independence.rs`) forbids importing
`lane_capacity*`, `ROW_LANE_FACTOR_*`, `utilization_for` — because, in its own
words, *"importing one would make the meter reproduce the engine's belief instead
of measuring it, and its agreement would be circular."*

Two wrong models cannot arbitrate each other. The meter can arbitrate both, in
~19s per fixture rather than the sim's 10–20 min.

**Load-bearing caveat.** The meter's *calibration* is asymmetric at the aggregate
level — "below plan" is believed, "at plan" is evidence of nothing
(`docs/meter-divergence.md` §2026-08-08, where the floor property was falsified on
the post-lift population). That is a claim about predicting factory output. Using
it as a **per-lane** oracle is a different and so-far unproven application. Phase 0
exists to establish it before anything leans on it.

### The units problem — the crux

The walkers emit per-lane **rates** (items/s) from a steady-state static solve. The
meter emits per-lane **occupancy/density** over ticks, in the time domain. The
comparison must be defined explicitly; a sloppy mapping produces a confident,
meaningless verdict.

Proposed observable: **items crossing a tile boundary per lane per unit time**, in
the meter's converged window (`Lane` already records "items that left this lane
through the run end, by tick of exit"). That is directly commensurate with a
walker's per-lane rate, where density is not.

Second, and separately: the RFC must say what distinguishes *"the model is blind
here"* from *"this belt really is empty in this regime"* — both present as 0.0/s.
Proposed discriminator: a tile is a **blind spot** if the meter observes items
crossing it while the walker reports 0.0/s. That is falsifiable per tile and needs
no judgement.

### Harvest before delete — the winner is not strictly better

`belt_flow` wins on the **rate** side: external-input seeding, the #519
consumption decrement, an iterative convergence pass. It **loses on the cap
side**: `belt_flow::check_lane_throughput` builds its `belt_name_map` from surface
belts and UG outputs only, with no splitter branch, so splitter tiles fall through
to the `"transport-belt"` fallback and are compared against a *yellow* cap
(7.5/s per lane) on red and blue layouts. `belt_structural`'s version maps them via
`splitter_to_surface_tier` and covers `splitter_second_tile`.

Measured consequence: a splitter-aware triage counts 1,563 over-cap readings where
the check as written reports 3,594 — consistent with roughly 57% of `belt_flow`'s
over-cap reports being this cap-side artifact. (The two counts come from different
code paths, so treat the ratio as indicative, not exact; Phase 1 pins it.)

So the sequencing is **harvest, then delete** — port the splitter tier mapping onto
the surviving check *before* removing `belt_structural`. Naming this explicitly
because "we picked the better model" is precisely the framing under which it gets
dropped.

### Known artifact classes in the surviving model

1. **Rate-side runaway.** A two-tile cycle (`di-row:copper-cable` ↔
   `ghost:flow:...:ret`) reaches **5730/s and 5735/s** on a yellow belt, against
   the convergence pass's own documented invariant that splitters damp cycle gain
   by 0.5 per pass. Bounded by `budget = 3 * segment_count`
   (`belt_flow.rs:2610`) — large-but-finite, not divergent. Observed on
   `logistic-science-pack@5/s` / am2 / yellow / `di=Forced`, tiles (44,30) and
   (45,30), which carry segments `di-row:copper-cable:electronic-circuit` and
   `ghost:flow:electronic-circuit:3:ret:30`. Reproduce by calling
   `belt_flow::compute_lane_rates` on that layout and reading those two tiles;
   the probe used is local-only, since `crates/core/examples/` is gitignored.
   **Measured share: 32 of 1,563 over-cap readings (2.0%).** Fixing this alone
   does *not* shrink the blast radius.
2. **Cap-side splitter mapping.** Above. Fixed by harvest.
3. **Untagged `carries`.** 933 of 1,563 over-cap readings sit on tiles with no
   `carries` attribution, so neither the item guard nor the triage can classify
   them. Same blind spot `docs/validator-trust.md` already records for
   `row-input-belt-margin`; this is a third consumer of it.

### Shape of the diff

- `crates/core/src/validate/belt_flow.rs` — fix the cycle damping; adopt the
  splitter tier mapping in `check_lane_throughput`.
- `crates/core/src/validate/belt_structural.rs` — delete `compute_lane_rates`,
  `classify_belt_feeders`, `FeedType`, `check_lane_throughput`, and their tests.
- `crates/core/src/validate/mod.rs` — dispatch `belt_flow::check_lane_throughput`
  (currently line 1079).
- `crates/core/src/bus/template_validate.rs` — unchanged (already on `belt_flow`).
- New: a shadow-mode divergence log, removed before final landing.

**Other consumers the delete breaks** — enumerated because "delete the loser"
reads as a two-file change and is not:

- `crates/core/tests/e2e.rs:3071` — the stress scoreboard registers
  `("lane_throughput", … belt_structural::check_lane_throughput …)`. It must be
  re-pointed, **and its baselines re-blessed**: the flip moves this scoreboard from
  the model that fires on 5 of 504 layouts to the one that fires on 176. Treat any
  scoreboard delta as expected-but-must-be-explained, per
  `SPAGHETTIO_STRESS_GOLDEN=check/bless`.
- Rustdoc cross-references at `crates/core/src/models.rs:253` and
  `crates/core/src/validate/inserters.rs:976` both name
  `belt_structural::check_lane_throughput` and would dangle.

### Alternatives considered and rejected

- **Keep both as differential oracles** (the RFC-065 pattern, which caught real
  drift for `belt_detour`). Rejected by owner call 2026-08-09: the meter already
  supplies the independent second opinion, and keeping two half-broken walkers to
  check each other is the situation this work exists to end.
- **Fix `belt_structural` instead** (add seeding + decrement + convergence).
  Rejected: that is re-implementing `belt_flow`'s three features to avoid porting
  `belt_structural`'s one.
- **Arbitrate the walkers against each other.** Rejected: neither is trustworthy,
  so agreement would prove nothing and disagreement could not be adjudicated.

## Kill criteria

- **Kill if Phase 0 cannot anchor the meter's per-lane readings.** If the meter's
  per-lane crossing counts cannot be reconciled with a sim observation on the #607
  fixture (where per-lane state is known: near lane 0/4, far lane 4/4), there is no
  oracle and the rest of this RFC is unfounded. Stop and write up the gap rather
  than falling back to walker-vs-walker.
- **Kill if the oracle cannot separate the two models.** If, on the arbitration
  corpus, both walkers' per-lane predictions are equally far from the meter's, the
  deficit is elsewhere (§2b zero-headroom) and unifying the walkers is orthogonal
  to it. Say so and stop rather than widening scope.
- **Kill if the fixed winner changes no candidate selection AND surfaces no new
  true positive** on the corpus. Then this is a tidy-up, not a correctness fix:
  descope to a straight deduplication with no behavioural claim, and drop the
  staged rollout.
- **Kill (descope to report-only) if the surviving check's new Errors cannot be
  sim-confirmed on at least one newly-flagged layout.** An Error-severity check
  gaining firing power without an anchor is how #519's inverted ranking happened.
- **Kill if post-fix runtime of the full validator regresses >2× on the e2e
  corpus.** The convergence pass is iterative and the budget is `3 ×
  segment_count`; a correctness fix that makes every layout build materially
  slower is not obviously a win.

## Verification plan

Per [`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes),
and note step 2's rule: the validator is not ground truth, and *"a check going
quiet is not evidence the problem is fixed."* Since this RFC's whole subject is a
check that went quiet, every claim below is a count, not a sample.

1. **Full e2e suite green** — `cargo test --manifest-path crates/core/Cargo.toml`.
   Counts quoted from one clean invocation, never summed across runs. The stress
   scoreboard needs an explicit re-bless at the flip (Phase 3), not a silent one:
   the `lane_throughput` row is expected to move from the 5-of-504 model to the
   176-of-504 model, and every delta should be attributable to a divergence
   already triaged in Phase 2's shadow log.
2. **Per-lane oracle anchor (Phase 0)** — the #607 fixture, meter per-lane
   crossings vs the known sim per-lane state.
3. **Arbitration table** — for each corpus fixture, both walkers' per-lane
   predictions against the meter's, with a blind-spot count (meter sees crossings,
   walker says 0.0/s) per model. This is the document's central evidence.
4. **Blast-radius sweep** — `--di off` / `--di forced` across configs, before and
   after, reporting *positioned issues* per instance rather than counts in a
   message (`docs/validator-reporting.md`).
5. **Shadow-mode divergence log** — both models computed, only the incumbent
   acting, over the corpus; the divergence set triaged before the flip.
6. **Sim anchor** — at least one newly-flagged layout: headless PASS, converged,
   kit-clean, drift ~0. Deep chains get a long `--warmup` (`docs/status.md`
   §"Default warmup is too short for deep chains").
7. **Clippy + WASM build** — checks, not nits.

## Phasing

- **Phase 0 — anchor the oracle.** Establish the meter's per-lane readings against
  sim ground truth on the #607 fixture. Gates everything else; first kill criterion
  lives here.
- **Phase 1 — arbitrate.** Build the arbitration table across the corpus. Pin the
  cap-side artifact share properly. Output is evidence, no engine change.
- **Phase 2 — fix the winner.** Cycle damping + harvest the splitter tier mapping.
  Shadow mode only; nothing steers selection.
- **Phase 3 — flip and delete.** Dispatch the survivor, delete the loser, sim-anchor
  before selection is allowed to consume the new firings.
- **Phase 4 (optional) — the untagged-`carries` hole**, if Phase 1 shows it
  dominates the residual. May split to its own issue.

## Decision log

- *2026-08-09 — opened. Grew out of #609, which was rescoped away from "one
  lane-loading model called by every check" after that issue's lead justification
  was measured and found to have zero population (2,898 layouts, 1,149,278
  belt-in tiles, no perpendicular feeds of any kind). Evidence in #609.*
- *2026-08-09 — owner call: **fix the winner and keep only the winner**; do not
  retain the loser as a differential oracle (RFC-065 pattern explicitly
  considered and rejected). Oracle is the meter.*
- *2026-08-09 — pre-RFC measurement changed the shape of this document. Triage of
  `belt_flow`'s over-cap readings found only **2.0% (32/1,563)** are physically
  impossible artifacts of the cycle runaway; 598 are plausible and 933 are
  unclassifiable. Fixing the cycle bug therefore does NOT reduce the blast radius
  to something small, so the staged rollout (Phase 2 shadow mode) is required
  rather than optional.*
- *2026-08-09 — same measurement surfaced the cap-side splitter-mapping defect in
  `belt_flow::check_lane_throughput`, i.e. the winner is not strictly better than
  the loser. Added "harvest before delete" to the design.*
- *2026-08-09 — review pass (#615, `second-opinion`) landed five findings, all
  valid, all fixed in the same PR. The material one: the original draft rested its
  "named, parked suspect" argument on `handoff-meter-as-gate-2026-08-07.md`, which
  is **not committed** — an untracked local file. Re-pointed to `docs/status.md`
  (post-lift 92.1% / 90.7%, "a real ~8-10% residual remains on both") and
  `docs/rate-stamp-semantics.md` (same two-walker split, same "arbitrate them"
  conclusion), so nothing load-bearing depends on an unreadable source. Also fixed:
  a stale `mod.rs:941-942` dispatch citation (correct line is 1079 — the original
  was read from a shared checkout that was both behind `origin/main` and carrying
  another session's staged edit to that file, which is exactly the "docs from the
  PR base, not the live checkout" trap); the segment breakdown not summing to its
  own total (top-5 presented as if complete — now all ten, summing to 112,407); an
  unreproducible repro pointing at a gitignored probe; and the delete plan omitting
  `e2e.rs:3071` plus two rustdoc cross-refs, and the scoreboard re-bless the flip
  requires.*
