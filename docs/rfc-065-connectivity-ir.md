# RFC-065: Connectivity IR — a derived topology lens for `LayoutResult`

Status: Phase 0 in progress (2026-08-04). Registry: `docs/rfcs.md` RFC-065.

## Summary

Give the layout artifact an explicit, **derived** representation of its
connectivity — which entity feeds which, through what mechanism — plus a
record-integrity pass that cross-checks the positional *records*
(`effective_rows`, coordinate-bearing `segment_id`s, `power_wires`) against
the geometry they claim to describe. Today none of this is represented:
`LayoutResult.entities` is a flat list, and every consumer (≈40 validator
checks, the compaction/fold transforms, the game itself) independently
re-derives connectivity from tile adjacency at read time. That is the root
cause of the transform-breakage class documented across RFC-055/056/057/058/
063/064: a transform cannot "update the links" because there are none — it
silently *re-derives different links*, and geometry-only validation cannot
tell "same topology, relocated" from "different topology".

The IR is a **lens, not a store**: `derive_connectivity(&LayoutResult)` is a
pure function; nothing is added to the serialized artifact, so the staleness
problem this RFC exists to kill cannot be recreated by the IR itself.

This RFC's implementation scope is **Phase 0 only** (module + parity +
integrity + the one live bug it catches). Phases 1–2 are enumerated and
pre-scoped; Phases 3–4 (movable components, port-to-port routing) are
explicitly **not** authorized here and get their own RFCs gated on Phase 0–2
evidence.

## Motivation

Three concrete, current problems, all instances of one structural fact.

**1. A live bug, reproducible today.** The shipped `compact_layout` path
(`?compact=1`, and every `fold_layout` run, which always compacts first)
rewrites entity coordinates in `strip_empty_columns` / `strip_empty_rows` /
`collapse_vertical_cut` / `collapse_horizontal_cut`
(`crates/core/src/bus/compaction.rs`), remapping `boundary_inputs`,
`boundary_outputs` and `surplus_exits` — but **never `effective_rows`**
(zero mentions in the file). `validate::resolve_row_spec_banded` resolves
"which sibling spec does this machine belong to" by `effective_rows` y-band
and **fails open** to the recipe-global spec on a miss, so every rate-shaped
verdict on a compacted layout (input-rate-delivery, inserter throughput,
lane rates) is potentially mis-attributed — including the `validate()`
calls inside `compact_validated_columns`/`_rows` that *admit each cut*.
Nothing in the 36 checks can notice, because nothing cross-checks records
against geometry. (The one exception, `check_boundary_record_integrity`,
exists precisely because a fold once shipped a validator-clean 0.00/s
factory — see RFC-057's decision log — and it guards only boundary
records.)

**2. The transform-breakage class.** The historical record (see
`docs/compaction-retro-2026-07.md` and the RFC-057/058 decision logs) is
dominated by moves that were *mechanically fine and semantically wrong*:
the fold that validated at exact control parity and produced 0.00/s in
Factorio; the fold that went 2 → 89 disconnected pole networks while both
sides reported `{"power": 1}`; `strip_empty_columns` clearing `regions`
and `trace` because nothing could remap them. The project's own rule
("any transform that relocates a boundary must move its record, and
geometry-only validation cannot be the admission gate for one") has no
enforcement mechanism beyond the single boundary check.

**3. Transform admissibility is generate-and-revalidate.** Every
fold/compaction candidate pays a full `validate()` whose cost scales with
entity count — the RFC-064 Phase 1 spike measured minutes of budget going
to two mega-chain fixtures that never produced an admissible fold, which is
why `fold_layout` ships behind a 6,000-entity threshold guard. A
"topology-preserving" transform has no representation of the topology it
preserves, so it cannot check preservation structurally; it can only
re-validate the world.

The structural fact underneath all three: **connectivity and attribution
live only in re-derivation.** This RFC gives them one canonical, testable
home.

## Design

New module `crates/core/src/connectivity.rs` (top-level sibling of
`validate`; no new deps; compiles under the `wasm` feature — pure data
code).

### Semantics come from the existing canon, not a 41st copy

The derivation **reuses** the `pub(crate)` primitives that
`validate::belt_flow` already exposes and `validate::belt_detour` already
consumes:

- `belt_flow::build_ug_pairs` — UG entrance tile → paired exit tile,
  reused **verbatim**: nearest same-direction ahead, dist > 1, with NO
  same-name filter. That is deliberately the primitive as it exists —
  weaker than `check_underground_belt_pairs`, which additionally requires
  name equality (as does the game, mechanics rule U5). The divergence
  matters only for interleaved mixed-tier runs on one axis, which the
  corpus never produces; adding a private stricter pairing here would be
  a fork of the semantics Phase 1 exists to unify. Recorded as a
  fidelity gap (review, 2026-08-04).
- `common::{dir_to_vec, inserter_reach, splitter_second_tile,
  entity_size, oriented_splitter_dims, is_*}` — geometry vocabulary

(`belt_flow::belt_dir_map_from`/`build_splitter_siblings` turned out
unnecessary: they are tile→direction/tile→tile maps, and the IR works at
entity level over its own tile→entity-index occupancy, which those maps
cannot express — see decision log 2026-08-04.)

Inserter convention (as in `belt_structural.rs` and `belt_detour.rs`):
pickup = `pos − dir·reach`, drop = `pos + dir·reach`.

### Types

```rust
/// Node = index into `layout.entities`. No new identity scheme.
pub struct ConnectivityGraph {
    /// Sorted, deduped. Directed: flow goes src → dst.
    pub edges: Vec<Edge>,
    /// Non-flowing structural facts worth surfacing (head-on belt
    /// contacts). Represented so a diff can see them; never traversed
    /// by flow reachability.
    pub conflicts: Vec<Conflict>,
    // + tile→node occupancy index and per-node adjacency ranges,
    //   private, for O(1) lookups.
}

pub struct Edge { pub src: usize, pub dst: usize, pub kind: EdgeKind }

pub enum EdgeKind {
    /// Belt/UG-exit surface flow onto the next belt-like entity —
    /// in-line or turning.
    BeltFlow,
    /// Perpendicular merge onto the side of a belt/UG tile.
    Sideload,
    /// UG entrance → its paired exit (the underground span).
    UgSpan,
    /// Belt-like tile feeding a splitter footprint tile.
    SplitterIn,
    /// Splitter footprint tile feeding the belt-like tile ahead.
    SplitterOut,
    /// Inserter hand: src is what it picks from (belt/machine),
    /// dst is the inserter.
    InserterPickup,
    /// Inserter hand: src is the inserter, dst is what it drops to.
    InserterDrop,
}
```

Edge rules (each mirrors an existing check's semantics; the parity gate
below is the enforcement):

- A belt-like entity at `t` facing `d` flows to the entity occupying
  `t + d` when that entity is belt-like and not facing directly back;
  facing directly back is recorded as a `Conflict::HeadOn`, never a flow
  edge (the game does not transfer there; `check_belt_junctions` errors
  it as an invalid angle).
- Flow arriving perpendicular to the receiver's direction is `Sideload`;
  in-line or turning is `BeltFlow`. Splitter-adjacent flow is
  `SplitterIn`/`SplitterOut` per footprint tile (both tiles of the 2-tile
  footprint participate; the splitter is one node).
- Paired UG entrances get a `UgSpan` edge to their exit (pairing from
  `build_ug_pairs`); an entrance with no pair simply has no `UgSpan`
  edge — the *absence* is the signal a diff sees.
- Inserters bind by reach-adjusted pickup/drop tiles against the
  occupancy index (machines occupy their full `machine_dims` footprint).
  A tile with no occupant yields no edge (again: absence is the signal).

### Topology diff — the transform-admissibility primitive

```rust
pub struct TopologyDiff {
    pub added: Vec<Edge>,
    pub removed: Vec<Edge>,
    pub added_conflicts: Vec<Conflict>,
    pub removed_conflicts: Vec<Conflict>,
}
pub fn diff(before: &ConnectivityGraph, after: &ConnectivityGraph) -> TopologyDiff
```

Because nodes are entity indices and edges carry no coordinates, the edge
set is **invariant under any rigid motion that preserves the entity list**
— which is exactly what "topology-preserving" means and what no code could
previously state. A transform that claims preservation can now assert
`diff(...).is_empty()` (or assert the diff is confined to its declared
seam) instead of — or before — paying a full `validate()`. Phase 0 only
builds and tests the primitive; wiring it into the fold/compaction inner
loop is Phase 2, with its own latency measurement.

Scope note: index-keyed identity means the diff is meaningful only for
entity-list-preserving transforms (translation, the strip/cut passes
before they splice, rotation of a segment in place). Transforms that
add/remove entities (UG splices, seam repairs) see those as node
additions/removals localized to the seam — still useful, but Phase 2 must
key its assertions accordingly. A coordinate-free *canonical* fingerprint
(graph hashing) is explicitly out of scope until a phase needs it.

### Record-integrity pass

```rust
pub fn check_record_integrity(layout: &LayoutResult) -> Vec<ValidationIssue>
```

Standalone in Phase 0 — **not** added to the `validate()` dispatch (see
K65-4). *Phase 1 slice 1 then wired it in as check #40; the phrasing
above records Phase 0's scope, not the current state.* One positioned
issue per instance, per `docs/validator-reporting.md` rule 1. Checks:

- **RI-1 `effective_rows` bands describe reality — harm-calibrated.**
  Every machine whose recipe has ≥1 `effective_rows` entry must have its
  FULL footprint inside some band for its recipe (membership alone
  under-detects; edge-exactness over-detects — see decision log), plus a
  foreign-band landing check, and every band must contain ≥1 machine of
  its recipe. Catches the live compaction staleness above precisely when
  it is harmful (when attribution changes).
- **RI-3 `power_wires` indices are sane.** Both endpoints in range and
  both poles (pole-ness via the canonical `power_wires::is_pole`). Reach
  re-verification stays owned by `check_pole_network_connectivity`; this
  is only the index-integrity slice no one owns — the exact staleness
  class the undergroundify pass once shipped (`bus/compaction.rs`'s
  wire-recompute comment is its record).
- **RI-2 (`segment_id` coordinate anchors) — built, then REMOVED when
  adversarial review tripped K65-1 on it.** The embedded coordinates are
  uniqueness keys, not position records: the router deliberately starts
  non-last tap runs one tile east of the embedded x
  (`ghost_router.rs` `start_x = x + 1`), junction zones re-stamp absorbed
  entities with bbox-corner `crossing:` ids, and no consumer reads the
  coordinates as coordinates. 62 false Errors on validator-green layouts
  (including in-corpus fixtures and freshly built default-path layouts).
  Segment-identity integrity returns in Phase 1 as graph-derived
  identity, not string archaeology. Full record in the decision log.

Boundary records are already owned by `check_boundary_record_integrity`;
RI does not duplicate it.

### The `effective_rows` fix (the one behavior change)

In `compaction.rs`: `strip_empty_rows` and `collapse_horizontal_cut` gain
the same `effective_rows` y-band remap that boundary records already get
(pure y-translations — bands remap exactly; `strip_empty_columns` /
`collapse_vertical_cut` are x-only and leave y-bands untouched).
`fold_snake` instead **clears** `effective_rows`: a fold cuts at x-columns
and relocates different x-ranges of the same y-band to different places,
so the per-row y-band encoding cannot represent the result — an honest
empty ledger (uniform fail-open fallback) beats a lying one, matching the
existing precedent of clearing `regions`. Graph/segment-based attribution
that survives folds is Phase 1+ material.

### Alternatives considered

- **Store the graph in `LayoutResult`.** Rejected: recreates the
  staleness class this RFC exists to kill; every transform would owe an
  update it can forget.
- **Extract the belt_flow primitives into the new module now** (move,
  not reuse). Rejected for Phase 0: churns validator internals inside a
  PR that must stay additive; extraction is Phase 1's first step, done
  check-by-check with verdict-identical gates.
- **Full graph canonicalization / coordinate-free hashing.** Deferred:
  no Phase 0–2 consumer needs it, and it is exactly the kind of
  speculative machinery the retro warns against.

## Kill criteria

- **K65-1 (zero false positives).** If `check_record_integrity` or the
  graph-anomaly scan reports any finding on the green corpus (the tier
  regression fixtures plus representative stress fixtures that validate
  with zero errors today) that cannot be confirmed as a genuine defect by
  an independent probe, the IR's model of the engine's semantics is wrong.
  Stop; do not tune thresholds to make it quiet — re-scope.
- **K65-2 (detection, not silence).** If the reconstructed failure
  fixtures — (a) compact-path stale `effective_rows` (harmful-class:
  attribution changes), (b) entity-list surgery with a stale
  `power_wires` graph (the documented undergroundify-wires class) — do
  not each produce ≥1 positioned finding from the integrity pass, the
  pass does not discriminate (the check-went-quiet mode,
  `docs/validator-reporting.md`). Stop. *(Class (b) was originally
  "stale segment anchors" — re-based when RI-2's invariant was falsified;
  see decision log.)*
- **K65-3 (cost).** If `derive_connectivity` + `diff` exceed 50 ms
  combined on the largest existing fixture (~19.5k entities,
  mega-chain-usp2raw scale) in a release-mode measurement, the IR cannot
  serve the fold/compaction inner loop that motivates Phase 2. Re-design
  the data layout before any Phase 1 work.
- **K65-4 (additive or nothing).** Phase 0 lands with the `validate()`
  dispatch untouched and all mainline goldens byte-identical (the
  `effective_rows` remap may only affect opt-in compact/fold paths; if it
  moves any default-path golden, that is a falsified assumption — stop
  and investigate before landing).

## Verification plan

Per `CLAUDE.md` § verification protocol:

1. Full suite: `cargo test --manifest-path crates/core/Cargo.toml` — all
   non-ignored tests green, mainline goldens unchanged (K65-4).
2. New tests, all in-module or `tests/connectivity_parity.rs`:
   - unit tests per edge kind (straight/turn/sideload/head-on/UG/splitter/
     inserter, incl. long-handed reach and machine-footprint binding);
   - parity: on representative green fixtures — tier1 gear, tier2 EC
     from ore, one fluid fixture, one DI fixture, PLUS (post-review) the
     shapes that falsified RI-2: tier4 AC@7 HorizontalStack (junction
     zones + multi-tap + HS rows, an in-corpus regression fixture) and
     tier4 AC@5-from-ore on the default path — zero integrity findings
     and zero graph anomalies (K65-1 evidence);
   - detection: the two reconstruction fixtures fire (K65-2 evidence);
   - regression: compacted layout's `effective_rows` bands contain their
     machines after the fix; fold clears the ledger.
3. `cargo clippy` clean on staged files; core compiles with `--features
   wasm`.
4. K65-3 measurement recorded in the decision log (release-mode, largest
   fixture available in the corpus).
5. Local adversarial review (independent agent re-running gates and
   probing claims) before commit-ready — this PR touches
   validator-adjacent semantics (`resolve_row_spec_banded` inputs), which
   is in the mandatory-review class.

## Phases (the arc this RFC opens)

- **Phase 0 (this RFC's implementation scope):** module + diff +
  integrity pass + `effective_rows` fix + gates above.
- **Phase 1 — single source of truth.** *Slice 1 landed 2026-08-04*:
  `build_ug_pairs` (now name-filtered, U5-true) and
  `build_splitter_siblings` are canonical in `connectivity`; `belt_flow`
  delegates, `belt_structural`'s private duplicate is deleted, and
  `check_underground_belt_pairs` consumes the canonical pairs (its inline
  loop — the fourth copy — is gone). `check_record_integrity` joined the
  `validate()` dispatch (check #40), so compaction/fold admission loops
  guard the records automatically. *Remaining*: `belt_dir_map_from`
  (deliberately not moved — its `skip_balancers` variant embeds
  sushi/balancer lane-walker policy, not geometry), migrating
  `resolve_row_spec_banded` to graph-derived identity, and per-check IR
  consumption (belt_detour first candidate) — one check per PR,
  verdict-identical on the corpus, per `docs/validator-reporting.md`.
- **Phase 2 — transform admissibility.** Fold/compaction inner loops
  assert topology preservation via `diff` before full validation;
  measure the latency win against the RFC-064 threshold guard
  (`FOLD_SEARCH_ENTITY_THRESHOLD` exists only because validation is the
  bottleneck).
- **Phase 3 — movable component (own RFC).** Promote
  `RigidIsland`/`IslandTerminal` to a first-class unit with directed
  ports and a D4 transform that composes `direction` × `mirror` ×
  splitter chirality × `fluid_ports` (the never-built `CellVariant`
  checklist from RFC-055; `zone_cache`'s D4 code is the seed). Gated on
  Phase 0–2 holding and on RFC-064's Phase 4 needing it.
- **Phase 4 — port-to-port fabric (own RFC).** The single-lane
  shared-trunk tier between arbitrarily-placed components — RFC-057's
  specified-but-never-built recommendation. Explicitly not funded here.

## Decision log

- **2026-08-04 — RFC opened.** Motivating evidence assembled from the
  compaction retro, RFC-055–064 decision logs, and a code audit of
  position-bearing state (session notes: two independent research passes
  over docs/git and over `bus/`+`validate/`). Root-cause framing:
  connectivity exists only as per-consumer re-derivation; records have no
  integrity owner.
- **2026-08-04 — Live bug confirmed before writing.** `effective_rows`
  has zero mentions in `compaction.rs` while every other positional
  record is remapped; `resolve_row_spec_banded` fails open. The cut
  admission loops (`compact_validated_columns/_rows`) validate candidates
  under the stale ledger. This is the RFC's reproducible-today anchor.
- **2026-08-04 — Derive, don't store.** The graph is a pure function of
  the artifact; nothing serialized. Rationale in Design § alternatives.
- **2026-08-04 — Reuse, don't extract (Phase 0).** Derivation imports
  `belt_flow`'s `pub(crate)` builders rather than moving them; extraction
  deferred to Phase 1 to keep this diff additive.
- **2026-08-04 — Node identity = entity index.** No new id scheme; diff
  is index-keyed and meaningful for entity-list-preserving transforms;
  canonical hashing deferred until a consumer exists.
- **2026-08-04 — Head-on contacts are conflicts, not flow.** Matches
  game behavior and `check_belt_junctions`'s invalid-angle error;
  `belt_detour`'s permissive treatment is a measurement convenience, not
  flow semantics.
- **2026-08-04 — RI-2 uses anchor-existence, not per-entity equality.**
  `tap:{item}:{x}:{y}` tags a whole run with its origin anchor
  (`ghost_router.rs:1637`); per-entity equality would false-positive on
  every healthy layout, violating K65-1 by construction.
- **2026-08-04 — Fold clears `effective_rows` rather than remapping.**
  A fold relocates x-ranges of a y-band independently; the per-row y-band
  encoding cannot express the result. Honest-empty over lying-full,
  precedent: `regions`/`trace` clearing in `strip_empty_columns`.
- **2026-08-04 — `belt_dir_map_from`/`build_splitter_siblings` not reused
  after all.** They map tiles to directions/sibling tiles; the IR needs
  tile → *entity index* (its own occupancy map, which also carries machine
  footprints those maps never had). Only `build_ug_pairs` carries
  semantics worth importing. The Design section's original three-primitive
  list was corrected in place.
- **2026-08-04 — RI-1 strengthened from membership to harm-calibrated
  (first draft falsified by its own detection test).** The membership
  check ("machine's `y` inside some own-recipe band") stayed silent on
  the real compact-path reconstruction: strip shifts are small, so
  machines remained inside their stale bands. Weakening the test was the
  wrong move; the right invariant fell out of asking when staleness is
  *harmful*: `resolve_row_spec_banded` mis-attributes exactly when
  resolution changes — a foreign-band landing or an all-bands exit.
  Ledger drift smaller than the row's internal margins resolves to the
  same spec and is functionally inert. RI-1 now checks full-footprint
  containment in an own-recipe band (true by construction for every
  placed row, so zero green-corpus false positives) plus foreign-band
  landing; K65-2(a) is read accordingly — detection targets harmful
  staleness, inert drift is explicitly not a defect.
- **2026-08-04 — Phase 0 gates run.** K65-1: parity green on all four
  fixtures (tier1 gear, tier2 EC-from-ore, fluid plastic, DI-Forced
  cable→EC) — zero anomalies, zero integrity findings. K65-2: both
  reconstruction classes fire (gross fold-class shift → RI-1; wholesale
  x-translation → RI-2), and the compact-path reconstruction
  discriminates on EC-from-ore (bands moved; stale ledger on compacted
  geometry caught; post-fix ledger clean). K65-3: synthetic serpentine at
  20,100 entities, release mode: `derive_connectivity` 2.91 ms,
  derive+diff 5.19 ms — ~10× inside the 50 ms bar
  (`connectivity::tests::bench_derive_and_diff_at_mega_chain_scale`).
  K65-4: `validate()` dispatch untouched; full-suite/golden verification
  recorded below.
- **2026-08-04 — Full-suite run: 2 failures, both PRE-EXISTING at HEAD
  (not this RFC's).** `tier4_advanced_circuit_from_ore_am2` (warnings
  `{input-rate-delivery: 12}` vs pinned `{belt-detour: 1,
  input-rate-delivery: 11}`) and `partition_strategy_scoreboard`
  (`AC@5/s plates yellow: P2 7 > 4`, alongside two *improvements* the
  gate says to tighten). Verified by A/B: `git stash -u` → identical
  failures with identical counts at pristine HEAD → pop. Almost
  certainly fallout of the #573-adjacent engine reverts (`b5d2b44`,
  `38416a5`) landing without a re-bless of these pins. Left for the
  owning session — re-pinning warning counts without understanding the
  revert's intended drift is exactly the check-went-quiet anti-pattern
  (`docs/validator-reporting.md`). Every other suite member passes with
  this RFC's diff applied; K65-4 holds (identical default-path behavior
  with and without the diff, failure-for-failure).
- **2026-08-04 — Adversarial review verdict: NOT COMMIT-READY as first
  written; K65-1 TRIPPED on RI-2 — check removed, not tuned.** The
  review's probe (9 additional green fixtures beyond the 4-fixture
  parity set) surfaced **62 `record-segment-anchor` Errors on
  validator-green layouts**, including the in-corpus tier4 AC@7
  HorizontalStack fixture (24), default-path AC@5-from-ore (5) and
  EC@20-from-ore (1), plus 4 on the engine's own sanctioned compaction
  output. Root causes in the router itself: non-last tap runs start at
  `x + 1` while the id embeds `x` (`ghost_router.rs`, ten lines above
  the line this log originally cited), and junction zones re-stamp
  absorbed entities with bbox-corner `crossing:` ids. The design-section
  claim "the id embeds the segment's origin anchor, stamped across every
  member" was **falsified** — the coordinates are uniqueness keys the
  router never maintains as positions, and no consumer reads them as
  positions. Per K65-1's own instruction, RI-2 was removed rather than
  re-fixtured (no id family carries a corpus-true anchor invariant), and
  K65-2's detection class (b) was re-based onto the historically-real
  `power_wires` staleness class (the undergroundify-wires bug, already
  documented in `bus/compaction.rs`). Also from the review: parity gate
  widened with the two fixture shapes that discriminate (junction zones,
  multi-tap, HorizontalStack; one in-corpus); `Splitter` dropped from
  inserter hand-binding classes (in-game impossible — a binding would
  bless a game-dead inserter; now an anomaly, with a unit pin); RI-3
  now uses the canonical `power_wires::is_pole` instead of a duplicate
  name list; fidelity gaps recorded in the module doc (UG pairing has no
  name filter; head-on conflicts carries-blind; `Sideload` onto a
  `UgExit` receiver over-models). Verdicts that HELD under attack:
  derivation semantics (mainstream cases), RI-1 across
  voider/kovarex/HS/refinery/DI/multi-row fixtures, the compaction
  remap math (incl. the exclusive-end argument), the admission-behavior
  change being strictly-more-correct and opt-in-confined, K65-3
  (re-measured 3.00/5.70 ms), and K65-4. The earlier "Phase 0 gates
  run" entry stands as the pre-review record; this entry supersedes its
  K65-1 line.
- **2026-08-04 — Phase 1 slice 1: one pairing derivation, dispatched
  integrity.** Before: FOUR underground-pairing implementations
  (`belt_flow::build_ug_pairs` direction-only; `belt_structural`'s
  private name-filtered copy; `check_underground_belt_pairs`'s inline
  name-filtered loop; connectivity's Phase 0 reuse of the first). After:
  ONE, canonical in `connectivity`, name-filtered — adopting the
  stricter U5-true semantics the check and `belt_structural` already
  used, which closes the review's recorded fidelity gap. The semantic
  delta (direction-only → name-filtered) reaches `belt_flow`'s six lane-
  walker call sites and `belt_detour`; it can differ only on interleaved
  mixed-tier undergrounds on one axis, which the engine never emits —
  gate is the full suite, verdict-identical. `check_underground_belt_
  pairs` now consumes the canonical pairs and keeps only its reporting
  (reach, interception, orphans); the refactor is order-preserving so
  issue lists are byte-identical. `check_record_integrity` joined the
  `validate()` dispatch — notable side effect: every `validate()` caller
  (including `accept_if_no_worse` and the cut loops) now rejects
  record-stale candidates by construction. CLAUDE.md's "36 functional
  checks" was stale at 39 before this; corrected to 40.
  `belt_dir_map_from` stays in `belt_flow` on purpose: its
  `skip_balancers` variant filters by sushi/balancer segment policy —
  validator policy, not geometry — and moving half a function invites
  drift. New pins: cross-tier non-pairing (U5) unit test; dispatched
  stale-ledger detection through the public `validate()`.
- **2026-08-04 — Phase 1 verification: independent-agent review DIED on
  the account's weekly usage limit mid-run; the equivalence probes were
  executed inline instead** (a process deviation, recorded per house
  rule; the PR-side second-opinion bot still reviews the pushed SHA).
  Probe results, all from a seeded-random UG-soup harness (temporary
  test, deleted after use) that reconstructed the pre-refactor pairing
  and check logic verbatim from git HEAD:
  (A) `check_underground_belt_pairs` old-vs-new across 400 soups (200
  same-tier, 200 mixed-tier), 3,863 issues compared field-by-field —
  **byte-identical, including order**. (B) the primitive's name-filter
  tightening is same-tier-inert (0/200 divergences) and bit on 32/200
  mixed-tier soups — the probe has discriminating power, and the corpus
  never builds mixed-tier interleavings. (C) the orphan-output
  `pairs.contains_key` form diverges from the old `used_outputs` form
  ONLY when an unpaired output physically overlaps a paired tile —
  reachable solely through entity overlap, itself a hard Error from
  `check_entity_overlaps`; accepted and documented rather than coded
  around. (D) dispatched-integrity cost: the naive band-populated scan
  measured **11.01 ms/call** at 20k entities × 50 bands — a real tax on
  fold-search candidate validation — and was restructured to collect
  machines once per call (index-per-recipe map serving both RI-1
  directions): **0.27 ms/call**, 40× better, negligible in any loop.
  Full suite with the slice applied: green except exactly the two
  pre-existing tier4/scoreboard failures (unchanged profiles).
- **2026-08-05 — PR #574 second-opinion bot review (of the Phase 0 SHA)
  triaged; two derivation fixes and four gate-hardenings taken.**
  Disposition per finding: (1) "integrity pass is production-dead code"
  — already resolved by the Phase 1 slice the bot hadn't seen
  (dispatched as check #40). (2) carries-blind HeadOn anomalies (3/3
  passes — the strongest finding): FIXED — `scan_graph_anomalies` now
  errors a head-on only when both sides carry the same item, mirroring
  `check_belt_junctions`'s carries-inequality skip exactly; the conflict
  stays recorded for `diff`. New pin: different-carries head-on is
  conflict-recorded, anomaly-silent. (3) gross-shift detection test
  can't discriminate the harm calibration: FIXED with a dedicated
  calibration-boundary pin (inert one-row drift stays clean;
  foreign-band landing and all-bands exit fire). (4) silent skip in the
  compact reconstruction test: now a hard failure, so a fixture that
  stops discriminating breaks the build instead of rotting the gate.
  (5) Sideload-onto-UgExit false flow edge: FIXED — no surface flow
  edge enters an exit tile (sideloading is an entrance-side mechanic,
  U7); an exit-side edge would have let Phase 2's `diff` bless
  game-impossible flow. (6) absence-only parity: the parity helper now
  asserts a positive structural floor (BeltFlow/InserterPickup/
  InserterDrop present) on every fixture. (7) narrow committed corpus:
  EC@20-from-ore added (the second RI-2-falsifying shape); voider/
  kovarex remain review-verified but uncommitted — open. Declined:
  none. Nits noted without action: power-wires detection robustness
  (deterministic on the pinned fixture), fold-admission semantics
  change (already logged 2026-08-04).
- **2026-08-05 — Bot review round 2 (of the Phase 1 SHA) triaged: one
  real transform gap fixed, two accuracy fixes, one dedupe; the major
  finding is the intended Phase 1 semantics, answered here.**
  Disposition: (major) "dispatching `check_record_integrity` is a
  non-additive behavior change" — correct, and it is Phase 1's entire
  point; K65-4's additive constraint bounded Phase 0, which held. The
  claimed over-strictness ("errors even when functionally inert via
  fail-open fallback") under-counts the consumer: `resolve_row_spec_
  banded` returns the BAND as well as the spec, and the rate walkers
  consume it as a row window, so exits and straddles are live drift
  even where spec identity survives; and neither can occur on an
  engine-built layout, so the green-corpus exposure is the committed
  parity set's coverage, not the check's semantics. (minor,
  apply_island_placement) REAL and fixed: the island transform
  translated machines in 2D and left `effective_rows` untouched — the
  exact class this RFC hunts, missing from its own transform list; it
  now clears the ledger like `fold_snake` (2D relocation cannot be
  band-represented). (minor, foreign-band message) correct — the
  message claimed foreign-spec adoption but `resolve_row_spec_banded`
  filters by recipe first and fails open to the recipe-global spec;
  message and calibration pin rephrased to name the true mechanism.
  (minor, straddle-vs-attribution) behavior kept, documentation
  tightened: the straddle case is deliberate (band-as-window
  consumers), impossible on engine output. (minor, name-filter has no
  structural guard) acknowledged residual — the tightening moves the
  primitive TOWARD game truth (U5), so any divergence it introduces is
  a divergence from modeling flow the game does not perform; a
  mixed-tier structural guard would be a new validator check and is
  Phase 1 backlog, not a blocker. (minor, fold fail-open residual)
  already logged 2026-08-04; graph-derived attribution remains the
  Phase 1 successor. (minor, stale module doc) fixed — module doc and
  this RFC's § record-integrity now state the dispatch wiring. (nit,
  `oriented_dims` duplicate) fixed properly: canonical
  `common::oriented_entity_dims` added; `connectivity::oriented_dims`,
  `compaction::entity_dims`, AND `strip_empty_columns`' third inline
  copy all delegate.
