# RFC-063: Compaction primitives — attacking the logistics floor

Registry: [`rfcs.md`](rfcs.md). Status: **Design (circulated for review)**.
Evidence base: [`compaction-retro-2026-07.md`](compaction-retro-2026-07.md),
which mines the decision logs of RFC-053 through RFC-061 and issues #135,
#456, #507, #519, #520, #526.

**2026-07-31 update: Phase A KILLED** (kill criterion 1 fired on the
Phase A0 feasibility probe, before SAT ran — see Phase A's gate and the
decision log). Phases B and C continue, at escalated bars (≥25% / −40.0%)
per kill criterion 1's own pre-registered escalation rule. Maintenance
residue from A0 tracked in #551.

## Summary

This RFC funds the next compaction arc as three primitive-level attacks on
the **logistics floor** — balancer/template footprint and per-band delivery
structure — under sim-anchored never-worse gates. It is the funded
follow-up to the 2026-07 compaction retrospective, whose record is
unambiguous: every gram of shipped density over the arc came from
candidates *inside* the decomposition search (RFC-053 direct insertion,
RFC-060 `HorizontalStack`), never from a post-pass over a finished layout
(RFC-057 shipped +38–250% larger candidates before being demoted to an
opt-in shape transform; RFC-058 was killed at −27.0% against a −33.0% bar).
Phase A regenerates/decomposes the balancer library primitives that #135
identified as the layout's actual waste in 2026-04 and that no session has
yet attempted. Phase B is a bounded spike on sharing one belt row between
two machine rows, the one wide-band reshaping RFC-058's census left
untested. Phase C re-runs RFC-058's own packing technique on DI-composed
input, which changes what its kill criterion measured and so is not
covered by that kill. This RFC explicitly does **not** fund another
whole-factory or post-pass repacking attempt; that direction is closed
until a primitive-level change moves the floor.

## Motivation

The retrospective's own numbers, restated here as the case for funding:

- **The floor was measured before this project's compaction work started
  and has never been attacked.** #135 (filed 2026-04-11): on
  `stress_electronic_circuit_30s_from_ore`, 34 of 37 adjacent-row bands
  have a 0-tile gap. The waste is three balancer bands of 15, 10 and 8
  tiles (150–280 entities each) — pre-generated `(n,m)` templates from
  `scripts/generate_balancer_library.py`, not scattered inter-row slack.
  Nothing has regenerated or decomposed those templates since.
- **The post-pass direction was tried at three granularities and lost at
  all three.** RFC-057 (machine/island granularity): first materialized
  candidates cost +38% to +250% bbox versus the bus they replaced —
  logistics outweighs machinery 6–8× (`sci1-ore`: 375 logistics entities
  vs the bus's 201). RFC-058 (row-band granularity): cleared every cheap
  phase (KC2 36.8% vs a 30% bar; the phase-3 spike at −35.9% vs a −33.0%
  bar) but fell to −27.0% once the real planner routed legally — six
  points under the bar, with the trajectory adverse as correctness
  increased (−44.0% → −34.6% → −27.0%). RFC-057's own decision log names
  the mechanism for the first failure and, unbuilt, recommends the
  granularity the second failure re-derived: *"do not pursue tree-based
  local manifolds further. Build the single-lane shared-trunk tier first
  ... the 2D placement itself is vindicated — it is the transport layer
  that is uneconomic."*
- **The floor is not a hypothesis.** It is the load-bearing structure a
  row already provides: one belt serving N machines, at roughly one belt
  tile per extra machine. Shattering that structure to place things more
  densely in 2D consistently spends more on the replacement delivery
  network than it saves in area. The only way past it is to make the
  delivery structure itself smaller — which is a primitive change (what
  gets built), not a placement change (where it goes).
- **Density that isn't sim-anchored isn't density.** #520 confirmed a
  validator-clean, 37-entities-denser DI layout that measured 2.52/s
  against a 5.00/s plan in Factorio — "density beat correctness because
  correctness was invisible." Every gate in this RFC inherits that lesson
  and is stated in sim terms, not validator terms.

Full timeline, the RFC-057 trajectory table, and the "don't re-fund" list
are in [`compaction-retro-2026-07.md`](compaction-retro-2026-07.md).

## Design

Three independent phases, each a primitive-level attack rather than a
post-pass. None depends on the others landing; Phase A is ordered first
because it is the oldest identified, most narrowly targeted lever.

### Phase A — shrink the balancer/template primitives (#135)

**Approach.** Regenerate the pre-baked `(n,m)` balancer templates in
`crates/core/src/bus/balancer_library.rs` via
`scripts/generate_balancer_library.py` with a tighter height budget, and/or
decompose the wide 1→M templates that currently dominate the three
oversized bands into row-gap-sized sub-balancers that pack against the
0-gap rows #135 measured. RFC-061 already made the library templates a
default-path cost (demand-skewed `LaneFamily`s), and RFC-057's own
hub-sensitivity measurement is a live warning on this approach: fan-four
vs fan-ten on the same military-science hub was 1,059 vs 2,928 entities —
generation choices on this primitive move entity count by 2–3× on their
own, so the regenerated library must be measured, not assumed better.

**Operational constraint.** `generate_balancer_library.py` requires
Factorio-SAT on `PATH`. This is a known dependency of the existing
tooling, not new to this RFC, but it gates who can run Phase A locally.

**Gate — STATUS: Phase A KILLED, 2026-07-31 (kill criterion 1 fired on
A0 evidence).** Original gate text ("the three oversized bands are 15,
10 and 8 tiles tall … the pre-registered bar is ≥25% combined reduction
… 33 → ≤ 24.75 tiles") is kept below for the record; it is no longer
active. The Phase A0 feasibility probe (see decision log) found the
premise underneath that bar — that row-height parity (~5-tall, "the
~5-tile height an ordinary machine row already achieves") is physically
approachable for these shapes — false: the best-known hand-optimized
design for `(4,5)`, the shape driving the worst band, is 11 tiles tall,
not ~5, and the realistically reachable combined gain against
verified best-known-practice references is ≈8% on the gate fixture and
≈6% on the holdout (bounding-box area, both measured against real
downloaded community blueprints, not the idealized calculation the
original bar used) — nowhere near ≥25%. Per RFC-058's own kill-criterion
discipline, which this RFC explicitly inherits ("Stop; do not re-tune the
packer" — RFC-058 KC1; reaffirmed at RFC-058's own kill, "the criterion's
own text says stop; do not re-tune"), a missed pre-registered bar is not
grounds to lower the bar to match what was measured. **Kill criterion 1
therefore fires: Phase A cannot beat its ≥25% bar, established by A0
evidence without running SAT.** See the decision log entry below for the
full derivation, the escalated Phase B/C bars this triggers, and the
maintenance residue spun off in its place.

*(Original gate text, retained for the record — no longer active:)* The
three oversized bands are 15, 10 and 8 tiles tall (33 tiles combined)
against a corpus where ordinary bands are almost always 5 tiles tall
(RFC-058's census). If regeneration/decomposition reaches row-height
parity — each balancer band shrinking toward the ~5-tile height an
ordinary machine row already achieves — the idealized ceiling is
`(33 − 15) / 33 = 54.5%`. Per the discipline RFC-058's own KC1 used (an
obstacle-free estimate is not a real-routing number; that RFC halved its
probe's −66.1% to set a −33.0% bar, and even that margin was thin), this
RFC halves the idealized ceiling: the pre-registered bar is ≥25% combined
reduction in the three named balancer bands' tile-height on
`stress_electronic_circuit_30s_from_ore` (33 → ≤24.75 tiles), measured
after real routing, not on the idealized calculation above. AND zero
regressions on the `balancer_lane_audit` / `KNOWN_IMBALANCED` tripwires.
AND any layout whose templates changed is sim-anchored (headless
Factorio, long `--warmup` per the deep-chain rule) before being called
never-worse — validator parity is not sufficient per #520.

### Phase B — wide-band reshaping: two machine rows sharing one belt row

**Status: KILLED 2026-07-31, on paper analysis, before a prototype was
built — see the decision log.** Ceiling 5.00–7.14% (row-kind-derived)
against the **governing ≥25% bar** (escalated from ≥15% same-day when
Phase A killed at A0, firing kill criterion 1) — misses both; the design
section's "wasted lane" premise below does not hold against the current
templates (`can_lane_split` already fills it for free). Kept as written
for the record of what was proposed.

**Approach.** RFC-058's phase-0 census found width-dominance is the
*entire* failure mode among ≥3-band candidates (5 of 19: `ec20-ore`,
`ac4-nauvis`, `ac5-nauvis`, `pu2-ore-hs`, `pu2.5-plates-hs`; widest bands
96–692 tiles, all smelter/assembler mega-rows). The obvious fix —
`max_per_row` capping, forcing extra sub-rows that each pay their own belt
overhead — was tested on 2026-07-30 and falsified: −0.9% bbox at zero
errors, or −5.3% only by introducing 10 errors. This RFC tests a different
reshaping RFC-058 explicitly left untried: two machine rows placed to
share a single central belt row between them, instead of each sub-row
carrying its own. That changes the cost structure `max_per_row` capping
paid for — the belt row that dominated each extra sub-row's overhead is
halved rather than duplicated.

**Gate — ESCALATED to ≥25%, 2026-07-31 (kill criterion 1 fired on Phase
A0 evidence; see Phase A's gate and the decision log for the trigger).**
A bounded spike (not a full implementation) on the five width-dominant
fixtures RFC-058's census named. Pre-registered before the spike runs,
because it must clear a bar set by an already-measured failed
alternative: originally **≥15% band-bbox reduction on those fixtures —
three times `max_per_row` capping's already-falsified −5.3% ceiling,
chosen so the new reshaping cannot be read as a rounding-noise win over
the technique it is meant to beat**. Kill criterion 1's own
pre-registered escalation rule raises this to **≥25%** — Phase A's own
missed bar — now that Phase A has failed to move the floor, on the
reasoning kill criterion 1 states: a reshaping technique should not ship
at a laxer standard than the more targeted, cheaper primitive that just
failed. Both bars are bounding-box area (already the metric this gate
used; the PR #547 gate refinements formally adopted at Phase A0 change
nothing here). With a never-worse, sim-anchored contract (no target that
validates clean today ships denser and slower).

### Phase C — DI-aware packing probe (successor named in #507)

**Approach.** RFC-058's `pu1-plate` gate fixture refused on the real
planner because the native pass is DI-free under the `Candidate` pattern:
copper-cable rides the bus at 81/s and trips the multi-lane refusal that
direct insertion normally removes. Composing DI rows *before* attempting
2D band packing changes what gets fed into the packer — fewer, larger-rate
single bands instead of many multi-lane ones — which is a different input
distribution to KC1, not a re-run of the same experiment. RFC-058's own
kill does not cover this case; it is explicitly recorded as future work
under its tracking issue (#507): *"closing the pu1 gap means packing the
DI candidate's rows — future work this RFC's phase-6 candidate wiring
should inherit, not a silent re-basing of the gate."*

**Gate — ESCALATED to −40.0%, 2026-07-31 (kill criterion 1 fired on Phase
A0 evidence; see Phase A's gate and the decision log for the trigger).**
An RFC-058-style throwaway spike (build nothing production-grade; answer
the question and discard the code), capped at one day of session cost.
Because this retests whether RFC-058's own *technique* clears its own
*bar* under different inputs — not a new technique — the correct bar to
re-clear was originally RFC-058's own: **KC1's −33.0% bounding-box-area
bar**, on whichever DI-composed gate fixtures the spike can build. Kill
criterion 1's own pre-registered escalation rule raises this to
**−40.0%** — a 7-point buffer above the bar RFC-058 already missed once —
now that a related primitive-level attempt (Phase A) has also failed to
move the floor, needing a wider margin to be persuasive than the
original. If DI composition does not change the packability picture (the
spike lands within noise of RFC-058's −27.0% result), that is
confirmatory evidence the floor is structural, not an input-distribution
artifact, and this RFC's Phase C closes without further attempts.

**Prerequisite/parallel, not in scope.** #526 (repairing the DI flagship
cell's belt-to-belt lift geometry, currently jammed or halved on its
canonical `copper-cable → electronic-circuit` coupling) is tracked
separately and is *not* part of this RFC's phases. Its outcome feeds
Phase C's ceiling directly: Phase C composes DI rows and packs them, so if
#526 is unresolved, Phase C inherits whatever DI cells #526 has not yet
fixed, and any packing result built on a still-broken DI cell is not
evidence about packing.

### Non-goals — the retro's don't-refund list, with citations

- **Tree-based local manifolds.** RFC-057's own decision log, 2026-07-30:
  *"do not pursue tree-based local manifolds further."* Balanced `(n,m)`
  trees are the wrong primitive for a corpus where almost every commodity
  needs exactly one lane (USP: 28 lanes total across all items, max 3 for
  any one item; military science 14 lanes, max 2).
- **Band packing as a post-pass.** RFC-058's kill criterion 1 fired; its
  own final entries name the tree router as "the option is spent after
  this iteration" and its concluding entry states the falsification
  directly: 2D band packing cannot hold ≥33% real-bbox saving under
  physically-legal single-lane trunk routing on its gate fixtures. Phase C
  above is not a re-opening of this — it is a narrowly scoped re-test on
  provably different inputs (DI-composed rows), explicitly bounded to one
  day and one bar.
- **Folding as a density lever.** RFC-057, 2026-07-29, reaffirmed
  2026-07-30: a firm ~20% routing-cost ceiling that survives every mirror
  variant, fold count and greedy fold-position search tried. Refused twice
  as a density lever; it remains a legitimate *shape* transform
  (`chain-mil5ore` 553×32 → 153×141 at plan in Factorio) and nothing here
  changes that status.
- **RFC-055 linear reordering.** Selected over RFC-056 on weighted
  distance (−16.3% to −39.6%) but its physical belt counts were mixed
  (−10.1% to −17.3% on three fixtures, +8.5% on USP), its Factorio gates
  were never adjudicated, and it never shipped. Superseded in practice by
  RFC-057's broader compaction work and RFC-057's own folding conclusion
  covering the same "reorder for shape" ground.
- **Meter expansion before the military-family error is attributed.**
  RFC-054's fast meter is ~20× cheaper than headless Factorio but tripped
  its own KC1: the EC family agrees to within 0.3–0.6pp, the military
  family is wrong by 57.8pp, and fluids read −100% on 7 of 12 reachable
  configs. Extending the meter's fidelity before that gap is explained
  risks building more surface on top of an uncalibrated instrument. None
  of this RFC's phases depend on the meter as an adjudicating gate — sim
  harness (headless Factorio) is the bar throughout, per kill criterion 2
  below.
- **Whole-factory repacking, generally.** The floor has now been measured
  at machine granularity (RFC-057), band granularity (RFC-058), and macro
  reorder granularity (RFC-055/056), and lost at all three. Nothing in
  this RFC funds a fourth attempt at that shape of solution; Phases A–C
  are deliberately primitive-level, not placement-level.

## Kill criteria

Pre-registered, at the RFC level (each phase also carries its own gate
above):

1. **STATUS: FIRED, 2026-07-31, on Phase A0 evidence (see decision
   log) — Phase A killed before running SAT; Phase B and C bars
   escalated below and in their own gate paragraphs above.**
   **If Phase A cannot beat its ≥25% bar, the logistics floor stands, and
   Phases B and C's bars tighten rather than run at their originally
   stated levels.** Concretely: Phase B's bar rises from ≥15% to ≥25% —
   Phase A's own missed bar, on the reasoning that a reshaping technique
   should not ship at a laxer standard than the more targeted, cheaper
   primitive that just failed to move the floor. Phase C's bar rises from
   RFC-058's −33.0% to **−40.0%** — a 7-point buffer above the bar RFC-058
   already missed once, because re-testing the same technique on
   different inputs after a related primitive-level attempt has failed
   needs a wider margin to be persuasive, not the same one.
2. **Any phase whose win exists only by the fast meter and not headless
   Factorio does not count.** RFC-054's KC1 tripped (military family
   57.8pp wrong; fluids −100% on 7/12 configs) — the meter is a screening
   tool for ranking candidates cheaply, never the adjudicating gate for a
   claimed win in this RFC.
3. **Never-worse means sim-anchored never-worse, per #520.** A layout
   that validates with zero errors and zero warnings is not evidence it
   works — #520's canonical case was exactly that, and measured 2.52/s
   against a 5.00/s plan. Every "never regresses" claim in this RFC is
   backed by a headless Factorio run at a warmup long enough to rule out
   buffer-fill transients (the deep-chain rule in `docs/status.md`), not
   by validator issue counts alone.

## Verification plan

Per the layout-engine protocol in
[`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Full e2e suite green** — `cargo test --manifest-path crates/core/Cargo.toml`,
  all non-ignored tests, after each phase that touches engine code.
- **Browser eyeball** on the fixtures each phase's gate names, before
  claiming a phase clears — a zero-warning layout with visibly
  disconnected belts is a validator bug, not a success.
- **Snapshot decode** (`SPAGHETTIO_DUMP_SNAPSHOTS=1`) for the specific
  defect class each phase is diagnosing, not just the aggregate warning
  count — per the nine (now ten, per #520) recorded instances of a
  quiet check concealing a live defect in
  [`validator-reporting.md`](validator-reporting.md).
- **Trace events** — Phase A's regenerated templates and Phase B's shared
  belt-row reshaping each emit a typed trace event carrying the
  before/after footprint, so a disappointing result is diagnosable
  without a debugger, matching RFC-058's `BandPackingPlanned` precedent.
- **Sim harness at long `--warmup`** on every fixture named in a phase's
  gate, for kill criterion 3 and each phase's own never-worse contract.
  Per `docs/status.md`'s deep-chain warmup note, the default warmup is
  known too short for multi-stage chains; any phase's gate result at
  default warmup is provisional until re-run long.
- **Clippy + WASM build** stay green through every phase; a compaction
  change that clippy-fails or breaks the WASM build is not done.

## Phasing

0. **This RFC circulated for review.** Design status; no engine code
   changes in this commit.
1. **Phase A — KILLED, 2026-07-31 (A0 probe, kill criterion 1 fired
   before SAT ran).** Was: regenerate/decompose the balancer library
   primitives against #135's named bands, ordered first as the oldest
   identified lever (2026-04-11) and narrowest scope. Maintenance residue
   spun off to #551.
2. **Phase B** — bounded spike on shared-belt-row wide-band reshaping.
   Gate escalated to ≥25% (was ≥15%), 2026-07-31, per kill criterion 1
   firing on Phase A's outcome.
3. **Phase C** — DI-aware packing throwaway spike, one day capped. Gate
   escalated to −40.0% (was −33.0%), 2026-07-31, per kill criterion 1
   firing on Phase A's outcome, and still gated on #526's DI-cell repair
   having landed enough that Phase C's packed candidates are built on
   working DI cells rather than inheriting #526's defect.
4. **Default-on decisions**, per phase, only after that phase's gate
   clears on sim evidence — no phase is promoted to default on validator
   parity alone (kill criterion 3).

Phases are independent; a phase's kill does not cancel the others, except
through the bar-tightening rule in kill criterion 1.

## Relationship to earlier RFCs

- **#135** is Phase A's origin and has been open, unattempted, since
  2026-04-11 — the retrospective's central finding is that this is the
  one lever the arc never actually tried.
- **RFC-057** supplies both the "logistics floor" diagnosis this RFC is
  built on and the specific mechanism (tree-based `(n,m)` local manifolds)
  Phase A must not repeat: Phase A changes the *static template library*
  used by the existing row/trunk architecture, not the runtime placement
  or manifold-routing architecture RFC-057 built and RFC-058 extended.
- **RFC-058** supplies Phase B's width-dominance census (5/19 fixtures,
  96–692-tile widest bands) and Phase C's exact scope gap (`pu1-plate`'s
  DI-free refusal). This RFC's non-goals close RFC-058's own direction
  (2D band packing as a post-pass) per its "the option is spent"
  conclusion; Phase C is a narrow, bounded, separately-gated exception,
  not a re-opening.
- **RFC-053, RFC-059, RFC-060, RFC-061** are the precedent this RFC
  continues: every one is a primitive- or candidate-level change inside
  the decomposition search, gated by sim-anchored never-worse contracts,
  and every one is where the arc's shipped density actually came from.
- **#520 / #526** establish that validator parity is not evidence of a
  working layout; this RFC's kill criterion 3 and every phase gate above
  inherit that discipline directly. #526 additionally feeds Phase C's
  ceiling, as noted in Phase C's design.
- **#519** is the sibling validator gap (lane-flux blind spot on
  sideload-fed taps) found during RFC-060's sim verification — a further
  reminder that a clean validator run is not sufficient evidence anywhere
  in this arc, including for phases in this RFC.

## Decision log

- **2026-07-31 — RFC opened as RFC-063.** Numbering coordinated with a
  concurrent sibling RFC: branch `rfc/multi-target-outputs` (PR #546)
  claimed RFC-062 for an unrelated multi-target-output proposal and left
  `docs/rfcs.md`'s "Next number" line at RFC-063 on that branch. Checked
  `docs/rfcs.md`, `gh pr list`, and `git ls-remote origin
  'refs/heads/rfc*'` before numbering; no other open PR or remote branch
  claimed RFC-062 or RFC-063 at that time. PR #546 merged
  (2026-07-31T20:43:56Z, commit `00490a03`) while this RFC was being
  written; this branch was rebased onto the result and its RFC-063 row
  now sits directly after RFC-062's in `docs/rfcs.md`, so no reconciling
  merge is needed. Status: Design, no phases started. Evidence base is
  `docs/compaction-retro-2026-07.md`, itself committed in this same PR.

- **2026-07-31 — Phase A0 feasibility probe (PR #547 review, item 2).**
  Executed the nearly-free probe the coordinating session's PR #547
  review comment proposed before funding SAT time: measure whether
  hand-optimized community balancers can already beat #135's named
  bands. Also formally adopts that review's item 1 (gate refinements)
  into Phase A's design section above. Full method and result:

  **1. Tooling verdict: Factorio-SAT is NOT installed in this
  environment — Phase A regeneration is BLOCKED until it is.**
  `external/` contains only `README.md` (setup instructions), no
  `external/factorio-sat` checkout; no `factorio-sat`/`fsat` binary on
  `PATH`; `pip show factorio-sat` finds nothing.
  `scripts/generate_balancer_library.py` needs a project-local
  `external/factorio-sat/.venv` (not a system package) built from
  `git clone https://github.com/R-O-C-K-E-T/Factorio-SAT.git
  external/factorio-sat && cd external/factorio-sat && uv venv .venv
  --python 3.12 && .venv/bin/python -m ensurepip --upgrade &&
  .venv/bin/python -m pip install --editable .` (documented in
  `external/README.md`). The prerequisite toolchain for that setup is
  already present on this machine (`uv` at `~/.local/bin/uv`, `python3.12`
  at `/usr/bin/python3.12`), so setup is a straightforward clone + venv
  build (network + a native `kissat` build; not attempted here — out of
  scope for a feasibility probe), not a missing-toolchain blocker. This
  is orthogonal to the feasibility question below, which was answered
  without running SAT locally.

  **2. Shape identification.** #135's issue body (2026-04-11) names only
  band tile-heights (15/10/8) and the recipe transitions they sit at
  (copper-plate→copper-cable→iron-plate), not exact `(n,m)` shapes.
  Re-ran `stress_electronic_circuit_30s_from_ore` with
  `SPAGHETTIO_DUMP_SNAPSHOTS=1` and decoded the `.fls` snapshot's trace
  events (`BalancerStamped`, `InterRowBand`) to get shapes directly —
  **the live 2026-07-31 fixture no longer matches #135's numbers.**
  `crates/core/tests/e2e.rs`'s own doc-comment baseline for both stress
  tests is stale in the same direction (`stress_electronic_circuit_30s_from_ore`:
  documented `entities=11232 … total_gap_tiles=33 … max_gap=15` vs.
  measured `entities=4966 … total_gap_tiles=34 … max_gap=17`;
  `stress_electronic_circuit_60s_red_from_ore`: documented `bands=3 …
  total_gap_tiles=22 … max_gap=12` vs. measured `bands=9 …
  total_gap_tiles=40 … max_gap=11`) — three months of RFC-057/058/060/061
  landing has moved these numbers and nobody re-baselined the comments.
  Live shapes, gate fixture (`stress_electronic_circuit_30s_from_ore`):

  | Band (item) | Shape (n,m) | Stamping | Height |
  |---|---|---|---|
  | copper-plate | (3,8) | direct template | 9 |
  | copper-cable | (8,10) | gcd-decomposed → 2×(4,5) | 17 |
  | iron-plate | (2,10) | gcd-decomposed → 2×(1,5) | 8 |

  Live shapes, holdout fixture (`stress_electronic_circuit_60s_red_from_ore`,
  named below):

  | Band (item) | Shape (n,m) | Stamping | Height |
  |---|---|---|---|
  | copper-plate | (3,6) | direct template | 9 |
  | copper-cable | (6,7) | direct template | 11 |
  | iron-plate | (2,7) | direct template | 8 |

  The decomposition itself (`family_stamp_plan`'s gcd search,
  `crates/core/src/bus/balancer.rs`) is *already* doing part of what
  Phase A's approach proposed ("decompose wide 1→M templates into
  row-gap-sized sub-balancers") — it landed as ordinary balancer-stamping
  machinery, not attributed to this RFC, so Phase A's actual remaining
  lever is narrower than the design section implies: regenerate the
  *sub-templates* the decomposition already selects, not invent
  decomposition from scratch.

  **3. Community-height comparison.** No in-repo blueprint corpus covers
  balancers (`crates/mining-cli`'s `blueprint-analyze` exists but the
  fixture corpus it's normally pointed at has none — searched for
  `corpus`/`fixture` blueprint files repo-wide, found none). Used
  Raynquist's balancer collection on FactorioBin (`KafN8H7L`), the
  standard community balancer book WebSearch surfaced repeatedly across
  independent queries. Downloaded the actual blueprint string for every
  shape below (`curl` on the FactorioBin CDN URL — not the page's prose,
  which one fetch already got subtly wrong) and measured it with
  `cargo run -p spaghettio_mining --bin blueprint-analyze`, our own tool,
  rather than trusting the site's stated dimensions:

  | Shape | Ours (W×H, area) | Community (W×H, area) | Area Δ | Height Δ | Source |
  |---|---|---|---|---|---|
  | (3,8) | 8×9, 72 | 8×10, 80 | **ours already smaller** (+11%) | ours already shorter | `KafN8H7L/26` |
  | (4,5) | 5×17, 85 | 7×11, 77 | community −9% | community −35% (17→11) | `KafN8H7L/97` |
  | (8,10)/(10,8) | 10×17†, 170 | 10×16, 160 | community −6% | community −6% (17→16) | `KafN8H7L/310` |
  | (1,5) atom‡ | 5×8, 40 | 5×7, 35 | community −12.5% | community −12.5% (8→7) | `KafN8H7L/138` |
  | (2,7) [holdout] | 7×8, 56 | 8×6, 48 | community −14% | not comparable (orientation) | `KafN8H7L/149` |
  | (3,6) [holdout] | 6×9, 54 | 7×7, 49 | community −9% | not comparable (orientation) | `KafN8H7L/90` |
  | (6,7) [holdout] | 10×11, 110 | 10×11, 110 | **parity** | parity | `KafN8H7L/115` |

  † our (8,10) is not a direct template — decomposed 2×(4,5), footprint
  is 2 stamps side by side. ‡ the (1,5) atom is what our (2,10) band
  decomposes to; no direct 2-to-10/10-to-2 design surfaced in the
  community collection within search budget, so the atom-level shape our
  own decomposition already uses is the fair comparison.

  Reading the table: **5 of 7 shapes have real, verified headroom (6–14%
  area) in a single hobbyist collection; 1 is at parity; 1 (`(3,8)`,
  the shape driving the *smallest* of the three bands) is a case where
  our current template already beats the best community reference
  found.** This is not the balancer-theoretic-minimum wall the review
  comment worried about — but it is also nowhere near the RFC's original
  54.5%-idealized / 25%-bar arithmetic, which was a height-only estimate.
  Recomputed on **area** (the review's own requested correction): swapping
  in the best verified reference per shape reduces the gate fixture's
  three named bands' combined template area from 322→296 tiles² (≈8.1%)
  and the holdout's from 220→207 tiles² (≈5.9%) — both used directly as
  this entry's revised bar (see the Phase A design section above).

  **Notable anomaly: `(4,5)` is very likely a stale SAT solve, not a
  proven floor.** `generate_balancer_library.py`'s own current search
  bounds for `base_h = max(n,m) = 5` sweep heights `[5, 6, 7, 8]`
  (`find_balancer`, the `elif base_h >= 5` branch) — height **17** is
  outside every height that codepath, unmodified, would even attempt
  today. The template must predate the current tuning of those bounds
  and has never been regenerated since. This is the single largest
  contributor to the worst band (copper-cable, height 17, the tallest of
  the three) and — independent of the community comparison — a strong
  candidate for "just re-run the existing script on this one shape and
  see what height it finds," before any script changes or full-library
  regeneration.

  **4. Gate refinements adopted from PR #547's review (items 1 and 2).**
  Both folded into the design section above, not left as
  decision-log-only notes: (a) the gate metric is whole-layout
  bounding-box area after real routing, with tile-height kept as the
  named mechanism rather than the pass/fail number — this was already
  true of Phase B's and Phase C's gates (both already bbox-area-based;
  only Phase A's was height-only) and now applies uniformly across all
  three phases' gates, escalated or not; (b) a holdout fixture
  (`stress_electronic_circuit_60s_red_from_ore`) was measured alongside
  the gate fixture throughout this probe — see the table above — so any
  future regeneration attempt inherits it as a standing check against
  overfitting a single fixture's specific shapes. Item 3 (Phase B's
  lane-contention pre-registration) is Phase B's concern, not actioned
  here — left for whoever picks up that phase.

  **5. Kill criterion 1 fires — Phase A killed at the funded-arc level,
  not re-tuned.** The ≥25% bar (§Kill criteria, item 1) was derived from
  an idealized ceiling that assumed row-height parity — shrinking each
  named band toward "the ~5-tile height an ordinary machine row already
  achieves" — is physically approachable. This probe's own measurements
  falsify that assumption directly: the best-known hand-optimized design
  found for `(4,5)`, the single largest contributor to the worst band, is
  **11 tiles tall, not ~5** (`KafN8H7L/97`, verified via
  `blueprint-analyze`), and the realistically reachable combined gain
  against verified best-known-practice references — substituting the best
  community design found for every improvable shape, keeping our own
  template wherever it already wins — is **≈8.1% (bounding-box area) on
  the gate fixture and ≈5.9% on the holdout**, not the ≥25% bar. This is
  not a case for re-tuning the bar downward to what was measured: RFC-058
  KC1's own text is explicit — *"Stop; do not re-tune the packer"* — and
  its closing decision-log entry applied that literally when its own
  measured result (−27.0%) missed its own bar (−33.0%): *"the criterion's
  own text says stop; do not re-tune."* RFC-063 explicitly inherits that
  discipline (Motivation: *"every gate in this RFC inherits that
  lesson"*; Non-goals cites RFC-058's kill directly). Accordingly: **kill
  criterion 1 fires, established by A0 evidence without running SAT.**
  Per kill criterion 1's own pre-registered escalation rule (already
  written into this RFC before A0 ran): **Phase B's bar rises from ≥15%
  to ≥25%; Phase C's bar rises from RFC-058's −33.0% to −40.0%** — both
  now stated explicitly in their own gate paragraphs above, dated
  2026-07-31.

  **6. Maintenance residue spun off — not a phase.** Killing Phase A at
  the arc-funding level is not a claim that nothing here is worth fixing.
  Two concrete, low-risk items fell out of the probe and are tracked in
  **#551** rather than left to rot in this decision log: (a) regenerate
  the `(4,5)` template specifically — its current height (17) sits
  outside the height range (`[5,6,7,8]`) `generate_balancer_library.py`'s
  own *current* search-bounds logic would even attempt for
  `base_h = max(4,5) = 5`, so 17 is very likely a stale artifact predating
  the current tuning, not a proven floor, and a fresh run under today's
  bounds is plausibly capable of matching or beating the 11-tall community
  reference on its own; (b) correct the stale `2026-04-11` doc-comment
  baselines on both stress tests in `crates/core/tests/e2e.rs` and #135's
  own stale numbers, all of which this probe found no longer match live
  measurement. Expected payoff (~8% gate-fixture bbox area, per this
  probe) is below the arc's step-change funding threshold, so this is
  opportunistic maintenance to pick up once Factorio-SAT is set up for any
  reason — not worth a dedicated session.

  **A0 verdict: Phase A KILLED at the funded-arc level (kill criterion 1
  fired). BLOCKED on tooling in this environment regardless** (moot for
  Phase A now, but recorded for #551's maintenance work and any future
  phase that needs SAT). Escalated bars now active: **Phase B ≥25%,
  Phase C −40.0%** (both dated 2026-07-31 in their own gate paragraphs).
  Maintenance residue tracked in **#551**, not funded as arc work.
- **2026-07-31 — kill criterion 1 fired: Phase A killed at the A0 probe
  stage (≥25% bar unreachable per measured community-best balancer
  ceilings), escalating Phase B's bar from ≥15% to ≥25%.** Per the RFC's
  own kill criterion 1, this happened *before* this spike's paper analysis
  concluded; the numbers below are evaluated against **≥25% as the
  governing bar**, with the original ≥15% carried alongside as context,
  not as the bar the verdict is judged against.

- **2026-07-31 — Phase B killed on paper analysis, before a prototype
  template was written.** The RFC's own escape hatch fired: "a paper
  analysis against `docs/factorio-mechanics.md` lane rules may kill this
  in an hour." It did, in about that time, via two independent findings —
  either alone caps the reshaping under both bars.

  **Finding 1 — the "wasted lane" the design section assumes does not
  exist in the current templates.** Phase B's motivating claim is that "the
  belt row that dominated each extra sub-row's overhead is halved rather
  than duplicated" — i.e. a row's own output belt normally wastes its near
  lane (I5: inserter drops fill only the far lane), so pairing two rows to
  fill both lanes from opposite sides reclaims it. But
  `crates/core/src/bus/placer.rs`'s `can_lane_split` (~line 656) already
  turns this on **unconditionally** whenever a row has ≥2 machines, for
  every row kind these fixtures use (`SingleInput`/`DualInput`/
  `TripleInput`/`FluidInput`). `templates.rs`'s `sideload_bridge` /
  `stamp_inline_bridge_a` fills the second lane via a 6-tile bridge stamped
  **inline** — `bridge_y` coincides with the existing output-inserter row,
  `output_y` with the existing output-belt row (module comment: "machines
  now pack tight with the bridge stamped inline") — at **zero row-height
  cost**. Every one of the five named fixtures' widest bands has 30–230+
  machines (96–692-tile range, this session's re-run below), so `count ≥
  2` always holds and the near lane is already claimed for free. There is
  no idle lane sitting in these rows for a sibling row to claim.

  **Finding 2 — the only belt that IS safe to share caps the geometry at
  ~5–7%, independent of Finding 1.** Reframing what the reshaping actually
  buys once Finding 1 is accounted for: not "reclaim an idle lane," but
  "delete one duplicate belt-tile-row when two same-recipe sub-rows are
  paired instead of stacked." Concretely, for a mirrored pair (row A's
  machines facing south into a shared output belt, row B's mirrored
  north into the same belt, each contributing the far lane on its own
  side per I5 — this is lane-safe, the same complementary-fill mechanism
  `sideload_bridge` already exploits, just via physical placement instead
  of a bridge): two independent rows of height `H` cost `2H` tile-rows;
  the merged pair costs `2H − 1` (one shared belt-row instead of two).
  Ceiling = `1/(2H)`. `RowKind`'s own doc comments give `H` per kind (all
  msz=3 — furnace/assembler, confirmed factorio-mechanics.md M7):
  `SingleInput` H=7 → **7.14%**, `DualInput` H=8 → **6.25%**,
  `TripleInput` H=9 → **5.56%**, `QuadInput` H=10 → **5.00%**. Width
  cancels in the area computation (splitting a row into narrower sub-rows
  doesn't change total tiles by itself — the same reason `max_per_row`
  capping alone measured near-zero, −0.9%, on 2026-07-30), so this ceiling
  applies directly to band-bbox area, not just to one row's height. Best
  case across every row kind in the corpus (`SingleInput`) is **7.14%**,
  under **half** the original ≥15% bar and **under a third of the
  escalated ≥25% bar** that governs per kill criterion 1 (Phase A killed
  at A0, above).

  **The first wall, answered directly per the spike's pre-registration:**
  output-side sharing (above) is lane-safe. Input-side sharing is not, and
  is excluded from the ceiling above. `max_machines_for_belt_both_lanes`'s
  own doc comment (`placer.rs` ~line 211) states it plainly: "the tap-off
  sideloads into the input belt, which (by B8) fills only one lane" — a
  row's local input belt is conventionally fed single-lane from the trunk
  regardless of whether its own inserters could in principle draw both
  lanes (I6). Sharing it between two rows would need the trunk tap-off
  itself widened to a two-lane feed (a separate, non-trivial change, not
  "share a belt row") and then resolve two independent inserter sets (one
  per row, opposite sides) competing for pickup from the same belt tile —
  unmodeled contention with no existing precedent in this codebase and
  squarely RFC-047 territory, as pre-registered. The nearest prior art,
  `di_cell.rs`'s producer/consumer row-sharing (~line 1696, "South face
  shares one row: reach-1 feeds from the inner belt, reach-2 outputs over
  it"), solves a different problem — one coupled item flowing
  producer→consumer on disjoint columns of the same row — not two
  independent inserter sets competing for the same belt's flow. A fully
  aggressive design that also chains input-sharing between successive
  row-pairs asymptotically approaches ~25% as row count → ∞, but only by
  accepting this unresolved risk, and the fixtures' realistic sub-row
  counts under a sane `max_per_row` split are nowhere near that limit.
  Not pursued.

  **Fixture numbers** (`probe_band_census_e2e_corpus`, re-run this
  session; band structure is host-geometry-relative per RFC-058's own
  2026-07-31 entry, so read as one machine's measurement):

  | fixture | bands | control bbox | widest band | width-dominant on this host? |
  |---|---:|---|---:|---|
  | `ec20-ore` | 6 | 144×49 | 144 | yes (refuses 3:1/3.5:1/4:1) |
  | `ac4-nauvis` | 5 | 97×45 | 97 | yes (refuses 3:1/3.5:1/4:1) |
  | `ac5-nauvis` | 6 | 121×53 | 121 | yes (refuses 3:1/3.5:1/4:1) |
  | `pu2-ore-hs` | 18 | 192×184 | 192 | yes (refuses 3:1/3.5:1/4:1) |
  | `pu2.5-plates-hs` | 14 | 73×159 | 73 | **no** — packs at 3:1, 77×40 (−73%) |

  **Ceiling vs both bars**, per row kind (structural — applies uniformly,
  not fixture-by-fixture, since the mechanism is "one belt-tile-row saved
  per merged pair" regardless of which fixture the pair sits in):

  | row kind | H (tile-rows) | ceiling `1/(2H)` | vs original ≥15% | vs escalated ≥25% |
  |---|---:|---:|---|---|
  | `SingleInput` (best case) | 7 | **7.14%** | misses by ~2.1× | misses by ~3.5× |
  | `DualInput` | 8 | 6.25% | misses by ~2.4× | misses by ~4.0× |
  | `TripleInput` | 9 | 5.56% | misses by ~2.7× | misses by ~4.5× |
  | `QuadInput` | 10 | 5.00% | misses by ~3.0× | misses by ~5.0× |

  Every row kind these fixtures use misses **both** bars; the escalation
  (≥15%→≥25%) doesn't change which side of the line the result falls on —
  it was already a clean miss at the original bar, just a closer one.

  Reported honestly, not silently dropped: `pu2.5-plates-hs` reproduced as
  width-dominant in RFC-058's original 2026-07-31 census but packs cleanly
  under ordinary band-packing on this host/run — the same host-sensitivity
  RFC-058 already documented (its own `ec10-ore` extracted 1 vs 3 bands
  across two machines at the same commit). Its exclusion here leaves 4/5
  fixtures as Phase B's live test population; the verdict is unaffected —
  the ceiling above is structural (row-kind height, not fixture-specific),
  so it applies uniformly and none of the 4 remaining fixtures can clear
  15% by construction.

  **Verdict: KILLED against the governing ≥25% bar** (escalated by kill
  criterion 1 when Phase A killed at A0, same date — see above). The
  ceiling (5.00–7.14%, row-kind-derived) would also have been a clean KILL
  against the original ≥15% bar on its own merits — this is not a case of
  the escalation flipping a would-be pass into a fail; the reshaping never
  cleared either line. No prototype template, no measurement harness, and
  no sim anchor were built — the RFC's own kill-criterion-3 sim-anchoring
  duty never triggers because nothing reached "validates clean" to anchor.
  Disposition: **KILLED**, no residue — Phase B's premise (an idle lane
  free for the taking) is false against the current template
  implementation, and the one savings mechanism that does exist once
  that's corrected (deleting one duplicate belt-tile-row per merged pair)
  cannot reach a third of the governing bar in the best case. With Phase A
  also killed (A0) and Phase B killed here, Phase C (DI-aware packing
  probe, still gated on #526's DI-cell repair per its own prerequisite) is
  RFC-063's only phase not yet resolved.
