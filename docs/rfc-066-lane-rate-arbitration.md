# RFC-066: Arbitrate the two lane-rate walkers against the meter

**Status: proposed 2026-08-09.** Tracking issue: #609. No *engine* code written —
no phase has started. The four `crates/core/examples/probe_*.rs` binaries listed
in the verification plan are committed evidence instruments and touch nothing the
engine dispatches.

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
| tile slots where the two walkers disagree >0.01/s | **134,702 / 231,704 (58.1%)** — both sides counted over tiles present in BOTH maps. A further **1,737** tiles exist in `belt_flow`'s map only (0 the other way), excluded from numerator and denominator alike, so this ratio is a floor for the same reason the blind-tile figure below is |
| tiles where the DISPATCHED model reads ~0.0/s and the other sees flow | **112,407** |
| — by segment (all, sums to the total) | 57,649 `row:`, 28,859 `trunk:`, 10,588 `ghost:`, 5,661 `di-row:`, 3,959 `crossing:`, 2,452 `feed:`, 1,981 `balancer:`, 872 `corr:`, 327 untagged, 59 `tapoff:` |
| `lane-throughput` firings, dispatched model | **5 of 504 layouts** |
| same check under the other model | **176 of 504 layouts**, *pre-cap-fix* — this is `belt_flow::check_lane_throughput` **as written**, including the splitter yellow-fallback firings the next section shows are ~57% artifact. The fixed winner's true count is lower; pinning it is what Phase 2 exists for |

Counting note: both checks push **one issue per over-cap lane**, so issue counts
are lane readings, not tiles — a tile over on both lanes counts twice. The
5-vs-176 *layout* comparison is unaffected, since both sides share the convention.

**Why the dispatched model reads zero.** `belt_flow::compute_lane_rates_impl`
seeds graph-source belts carrying external inputs (`belt_flow.rs:2623-2688`), and
says why in its own comment:

> *"without this seeding, rate propagation starts at 0 and every downstream
> consumer of an external input is incorrectly flagged as starved."*

`belt_structural::compute_lane_rates` has exactly one seeding site —
`belt_structural.rs:1033`, *"Seed lane injection rates from output inserters"* —
and no external-input onramp. An unseeded graph *source* propagates zeros to
everything downstream of it, which is every segment category on the input side.

**The primary justification stands alone and does not depend on any deficit.**
An `Error`-severity, selection-participating check is running on one of two
models that have never been arbitrated, and that model reads ~0.0/s across
roughly half the belt graph. `docs/rate-stamp-semantics.md` §"the disagreement is
unresolved and it matters" reaches the same place independently: it identifies the
same split, notes at its point 3 that the dispatched model *"reports **0** tiles
over capacity in every arm at every stack size"* where `belt_flow` flags the S=1
arms, and names **arbitrating the two
models** as the follow-up. (That doc cites the dispatch at `validate/mod.rs:939`;
at this RFC's base the line is 1079 — the code moved, the claim did not.)

**Secondary, and deliberately stated weakly: the open residual.** `docs/status.md`
records the two stress-EC fixtures re-measured post-lift at **92.1%** and
**90.7%** delivered, with *"a real ~8-10% residual remains on both"*, root cause
not chased. That residual is **not** claimed here as caused by the walker split —
`status.md` currently attributes it to zero-headroom integral machine counts (all
four of those stages are exactly zero-headroom), which is a competing explanation
this RFC does not displace. The third kill criterion exists precisely to settle
that: if both walkers predict those fixtures equally badly, the deficit is
elsewhere (`docs/status.md`, the zero-headroom integral-machine-count entry) and
this work is orthogonal to it.

**Explicitly NOT cited as motivation: `tier2_electronic_circuit`.** Its ~9–10%
was root-caused in `status.md` (2026-08-08, #607/#608) to the `di-bridge`
belt→belt transfer bank loading one lane only — ~21.4/s against 30/s of demand —
and once #608 credited that belt honestly, selection ships the bus-lane variant,
which measures **100.0% of plan** headless against the bridge's 90.9%. That entry
carries an explicit *"do not cite the paragraph below for this fixture"*. It is a
**solved** case.

It is worth keeping for a different reason, though: it is direct evidence **for
the oracle**. The meter caught that defect — nine of ten copper-cable machines
saturated, the stage able to make plan, the binding constraint elsewhere — which
is exactly the per-lane discrimination Phase 0 needs it to have.

A session handoff ([`handoff-meter-as-gate-2026-08-07.md`](handoff-meter-as-gate-2026-08-07.md))
covers similar ground and is where I first read the framing. It was untracked
when this RFC was reviewed — see the 2026-08-09 decision-log entry — and was
committed afterwards. Nothing here depends on it, and that has not changed: it
is a session note, not a source.

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
   (`belt_flow.rs:2610`) — large-but-finite, not divergent. Read that local name
   with care: `segment_count` is `belt_dir_map.len()`, a **belt-tile** count, and
   the comment above it (`belt_flow.rs:2596-2609`) contrasts exactly this — a
   futile `3 × distinct-segment-ids` against the workable `3 × belt_tiles`. The
   name is a wart in the source; this RFC quotes it verbatim rather than silently
   correcting it. Observed on
   `logistic-science-pack@5/s` / am2 / yellow / `di=Forced`, tiles (44,30) and
   (45,30), which carry segments `di-row:copper-cable:electronic-circuit` and
   `ghost:flow:electronic-circuit:3:ret:30`. Reproduce with
   `cargo run --manifest-path crates/core/Cargo.toml --example probe_walker_shape
   --release -- sci2`, committed alongside this RFC.
   **Measured share: 32 of 1,563 over-cap readings (2.0%)** — and that is a
   **lower bound**: the triage's conservation ceiling uses `count.ceil()`, which
   over-estimates supply and so under-counts what it can prove impossible. The
   remaining 1,531 are **unclassified**, not confirmed-real: 598 are merely *not
   provably impossible* (a lane carrying 90% of the factory's entire output of an
   item clears that bar) and 933 sit on tiles with no `carries` attribution. So
   fixing the cycle bug alone does not shrink the blast radius, and the blast
   radius itself is **unknown** rather than known-large. That is the argument for
   Phase 2's shadow mode: it is how the number gets established, not a mitigation
   for one already measured.
2. **Cap-side splitter mapping.** Above. Fixed by harvest.
3. **Untagged `carries`.** 933 of 1,563 over-cap readings sit on tiles with no
   `carries` attribution, so neither the item guard nor the triage can classify
   them. Same blind spot `docs/validator-trust.md` already records for
   `row-input-belt-margin`; this is a third consumer of it.

### Known limits of the B8 instrument

Stated rather than implied. Two blind spots survive in `probe_b8_modes.rs`, both
in the under-counting direction, and both left unfixed as vacuous **given** the
measured result rather than assumed vacuous in advance:

- **B8 into a UG output is not detectable.** `straight` is set only by a
  registered neighbour behind the tile; the tile behind a UG output lies in the
  tunnel and never enters the feeder index, so a perpendicular feed coexisting
  with the underground straight feed would land in a benign bucket. The probe's
  header claims UG-output coverage — that claim is overstated.
- **A splitter inside a belt-in run is never scanned as a run tile** (the run
  filter is `is_surface_belt || is_ug_belt`), although splitters *are* registered
  as feeders.

Both are vacuous only because the sweep finds **no perpendicular feed of any kind**
into these runs: with zero perpendicular feeders present, neither gap has anything
to misclassify. If any future run reports a non-zero perpendicular count, both
must be closed before the result is quoted. Recorded this way because two earlier
under-counting defects in this same probe were each rationalised as probably
harmless, and both needed a full re-run once found.

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
  re-pointed, **and its baselines re-derived**: the flip moves this scoreboard from
  the model that fires on 5 of 504 layouts to the one that fires on 176. Treat any
  scoreboard delta as expected-but-must-be-explained. (2026-08-15, #632 B7: the
  committed-golden `check`/`bless` flow this bullet originally named is deleted —
  the baselines to update are the always-on `StressBaseline` structs in `e2e.rs`
  and the warning-pin goldens, with before/after `STRESSGOLD` hash captures as
  the byte-stability record.)
- Rustdoc cross-references at `crates/core/src/models.rs:253` and
  `crates/core/src/validate/inserters.rs:976` both name
  `belt_structural::check_lane_throughput` as intra-doc links and would dangle.
- `crates/core/src/validate/mod.rs:432` — the `resolve_row_spec` doc comment
  lists `belt_structural::compute_lane_rates` among the checks sharing it. This
  one is a **plain-backtick** reference, not an intra-doc link, so rustdoc will
  NOT flag it: it rots silently and must be cleaned by hand.
- `crates/core/src/validate/inserters.rs:978-982` — **already stale today, and it
  contradicts this RFC.** The prose says `belt_flow::check_input_rate_delivery`'s
  *"lane-rate propagation never subtracts what upstream machines on the same belt
  have already consumed"*. True before #519; `belt_flow.rs:2512` now documents the
  forward consumption decrement, one of the three features this RFC credits
  `belt_flow` with. Whoever re-points line 976 must rewrite this paragraph, or a
  reader checking the source finds the repo contradicting the RFC's premise.
- `crates/core/examples/probe_walkers.rs:73` and
  `crates/core/examples/probe_walker_shape.rs:25` — **the probes this RFC commits
  as its own reproducibility guarantee call `belt_structural::compute_lane_rates`
  directly.** Deleting it stops them compiling, so an implementer following this
  plan breaks the very instruments the reproduction table below tells them to run.
  They must be ported (to compare the survivor against the meter) or deleted with
  the model. Called out because it is the most self-inflicted item on this list.
- `crates/core/src/validate/inserters.rs:1041` — an unqualified
  `check_lane_throughput` backtick. It will not dangle (a function of that name
  survives in `belt_flow`), but its referent silently changes, so the surrounding
  claim needs re-reading rather than a mechanical rename.

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
  fixture — whose per-lane state is recorded in committed source at
  `crates/core/src/validate/inserters.rs:1355-1356`: *"the near lane reads `0/4`
  along the entire run while the far lane saturates at `4/4`, and the layout
  delivers 90.9% of plan while validating clean"* — there is no oracle and the
  rest of this RFC is unfounded. Stop and write up the gap rather
  than falling back to walker-vs-walker.
- **Kill if the oracle cannot separate the two models.** If, on the arbitration
  corpus, both walkers' per-lane predictions are equally far from the meter's, the
  deficit is elsewhere — `docs/status.md`'s zero-headroom integral-machine-count
  entry is the standing competing explanation — and unifying the walkers is
  orthogonal to it. Say so and stop rather than widening scope.
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

**Reproducing the numbers in this RFC.** Every corpus aggregate above comes from a
probe committed alongside it, so a later implementer can re-run them and detect
corpus drift before building the arbitration table. `crates/core/examples/` is
gitignored by default; these four are whitelisted explicitly, on the same
reasoning the `sim_export.rs` exception records:

| figure | probe |
|---|---|
| 58% tile-slot disagreement; the 112,407 blind tiles and their segment breakdown; 5-of-504 vs 176-of-504 | `probe_walkers.rs` |
| 2.0% impossible / 598 plausible / 933 unclassifiable over-cap triage | `probe_overcap_triage.rs` |
| 2,898 layouts, 1,154,966 belt-in tiles, zero B8 (the result that rescoped #609) | `probe_b8_modes.rs` |
| the 5730/5735 cycle runaway on `sci2` | `probe_walker_shape.rs` |

Run as `cargo run --manifest-path crates/core/Cargo.toml --example <name>
--release`. Note `probe_walkers.rs` reads issues out of `validate()`'s **`Err`**
variant deliberately — that call returns `Err` *carrying* the issues whenever any
fire, so the natural `unwrap_or(0)` reports zero findings for every layout that
has them. An earlier draft of this RFC quoted a figure produced that way.

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
  was measured and found to have zero population (2,898 layouts, 1,155,086
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
  is **not committed** — an untracked local file. [**Annotation 2026-08-10 (#622):**
  that file has since been committed to `docs/`, so it is readable now. The finding
  and the re-pointing below both stand — the argument should not have rested on it
  either way, and nothing was reverted.] Re-pointed to `docs/status.md`
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
- *2026-08-09 — second review pass on #615 landed two more findings, both valid.
  The substantive one: the motivation cited `tier2_electronic_circuit`'s ~9-10%
  residual as an unexplained shared constraint, but `docs/status.md` marks that
  attribution **superseded** (2026-08-08, #607/#608) — root-caused to the
  `di-bridge` single-lane transfer bank, and the re-ranked layout now measures
  100.0% of plan. The entry carries an explicit "do not cite the paragraph below
  for this fixture", which this RFC did. Restructured the motivation as a result:
  the primary justification (an Error-severity selection-participating check
  running on an unarbitrated model that reads ~0.0/s across half the graph) stands
  alone and needs no deficit at all; the stress-EC residual is demoted to
  secondary and explicitly NOT claimed as caused by the walker split, since
  `status.md` attributes it to zero-headroom and this RFC does not displace that.
  tier2 is retained only as evidence FOR the oracle — the meter caught that
  defect, which is the per-lane discrimination Phase 0 depends on. Second finding:
  added `validate/mod.rs:432` to the consumers the delete breaks; unlike the two
  intra-doc links, it is a plain-backtick doc-comment reference that rustdoc will
  not flag, so it would rot silently.*
- *2026-08-09 — third review pass on #615: six findings, three upheld, three
  rejected on inspection. **Upheld:** (a) the corpus aggregates had no
  reproduction path — the four generating probes are now committed with
  `.gitignore` exceptions, closing the same "unverifiable source" class the first
  two passes hit; (b) `inserters.rs:978-982` is stale TODAY and contradicts this
  RFC, claiming `belt_flow` "never subtracts what upstream machines have already
  consumed" when `belt_flow.rs:2512` documents the #519 decrement this RFC credits
  it with; (c) a dangling "§2b" cross-reference inherited from the uncommitted
  handoff, re-pointed at `status.md`. **Rejected:** the #607 per-lane anchor was
  called uncommitted, but it is in tracked source at `inserters.rs:1355-1356` (now
  cited rather than asserted); a quote was called a paraphrase, but it is verbatim
  from `rate-stamp-semantics.md`'s point 3 (line reference added); and one finding
  cited `crates/core/src/validate/e2e.rs`, which does not exist. Committing the
  probes required making them clippy-clean — `-D warnings` is a gate once they are
  tracked, and they were not.*
- *2026-08-09 — fourth review pass on #615 found three MAJOR defects, two of them
  in `probe_b8_modes.rs` itself — the instrument behind the zero-B8 result that
  rescoped #609. It did not register a splitter's **second tile**, so a run head
  fed straight through that tile read as having no straight feeder and a real B8
  would have been downgraded to a benign B11 turn; and its run scan was filtered
  to `is_surface_belt`, so **underground belt-in tiles were never examined** at
  all, while the quoted tile count was presented as the whole population. Both
  defects bias toward UNDER-counting B8, i.e. toward the answer the probe
  reported. Third: the RFC still described the runaway probe as "local-only, since
  `crates/core/examples/` is gitignored" in the same commit that committed and
  whitelisted it, 120 lines above a table telling the reader to run it.*
- *2026-08-09 — probe corrected (splitter second tiles registered, UG run tiles
  scanned, B8 count now requires an item match, U7 UG-input feeds split out, and
  multi-perpendicular tiles no longer hidden in the benign bucket) and the sweep
  **re-run in full**. Result unchanged: 2,898 layouts, **1,154,966** belt-in run
  tiles (up from 1,149,278 — the difference is the UG coverage that was missing),
  **zero B8, zero B10, zero B11, zero U7, zero item-mismatch, zero
  uncategorised**. The rescope of #609 stands, now on an instrument whose two
  under-counting defects have been removed. Independently corroborated by the
  repo owner from design knowledge: the engine terminates taps along the run axis
  and does not side-feed a row input belt.*
- *2026-08-09 — self-audit of the two remaining probes, prompted by the above
  rather than by a reviewer. `probe_walkers.rs` is sound: its blind-tile count
  compares the two rate maps directly with no classification step to get wrong,
  and an independent agent reproduced it (112,405 vs 112,407). `probe_overcap_triage.rs`
  is structurally sound but its OUTPUT was over-read here: its conservation
  ceiling uses `count.ceil()`, an over-estimate, so **2.0% is a lower bound on
  artifacts**; and "plausible" only ever meant *not provably impossible*, not
  *confirmed real*. The honest split is 32 provably impossible and 1,531
  unclassified. The staged rollout is therefore justified because the blast radius
  is **unknown**, not because it is known to be large — which is a better reason
  for shadow mode, not a worse one. Corrected in §"Known artifact classes" and the
  Phase 2 rationale.*
- *2026-08-09 — fifth review pass caught that the splitter fix claimed in the
  entry above **was never applied**. The patch carrying it hit an assertion on a
  later edit and exited before writing the file, so the change was silently
  discarded — while the commit message and this log both asserted it had landed.
  The sweep published in between therefore still carried one of the two
  under-counting defects. Applied for real (`probe_b8_modes.rs:159`, verified by
  grep AFTER the edit rather than trusting the patch's exit status) and the sweep
  re-run a third time: 2,898 layouts, **1,154,966** belt-in run tiles, **zero B8**
  and zero in every other category. The result has now survived three instrument
  versions, two of which were defective in the direction of the answer they gave.*
- *2026-08-09 — corpus is NOT bit-reproducible: the same sweep counted 1,155,086
  tiles on one run and 1,154,966 on the next, a 0.01% drift with no code change
  between them, almost certainly the runtime SAT zone cache. Immaterial to a zero
  result, but it means Phase 1's arbitration corpus must be **pinned** (exported
  blueprints or `.fls` snapshots), not merely re-derived by re-running the probe.
  Recorded rather than smoothed over, since this RFC's reproduction section
  otherwise implies re-running gives identical numbers.*
- *2026-08-09 — three remaining minors fixed: the 112,407 blind-tile figure is a
  **floor** (tiles present in only one model's map are excluded from both sides,
  and a tile `belt_structural` omits entirely is the strongest blindness there
  is); `b8_item_matched` was a tautology once the B8 branch required an item match,
  so the misleading qualifier is gone; and `probe_walker_shape.rs` no longer
  panics on an empty lane map or a NaN rate, which matters now that it is a
  committed instrument rather than a scratch script.*
- *2026-08-09 — sixth review pass, eight findings. The one that mattered: the two
  probes this RFC commits **as its own reproducibility guarantee** call
  `belt_structural::compute_lane_rates` directly, so the delete plan breaks them —
  the most self-inflicted omission in the consumer list, now added. Also corrected:
  the 58% ratio mixed denominators (numerator over tiles in both maps, denominator
  over their union) and is now 134,702 / 231,704 with the 1,737 one-sided tiles
  called out; the "tiles" label on issue counts actually counts per-lane readings;
  and 176-of-504 is marked **pre-cap-fix**, since it includes the splitter-fallback
  firings this RFC separately shows are ~57% artifact. Two surviving instrument
  blind spots (B8 into a UG output; a splitter inside a run) are recorded as
  limitations rather than fixed — both vacuous **given** a measured zero
  perpendicular feeds of any kind, with the RFC now stating they must be closed
  before any non-zero perpendicular count is quoted. Judgement recorded so it can
  be checked rather than taken on trust.*
- *2026-08-09 — process note, because it bit twice in one session. Two patch
  scripts in this branch's history asserted on a later anchor and exited **before**
  writing the file, silently discarding edits the commit message then claimed had
  landed — once for the splitter fix (caught only by review, after a sweep had
  been published on the unfixed probe), once for this very entry's edits (caught
  because the second time I grepped for the effect instead of trusting the exit
  status). Any patch that reports success must be verified by grepping for its own
  effect before anything is written about it.*
