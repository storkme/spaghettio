# RFC-065: Connectivity IR — a derived topology lens for `LayoutResult`

Status: Active — Phase 0 landed; Phase 1 slice 1 landed; Phase 2 closed
by measurement (2026-08-05): the admission pre-filter is dead on both
paths — fold-side killed on the pre-registered ≥30% criterion
(Error-catchable share of rejected fold candidates measured at 0.83%),
cut-side default-off after adversarial review falsified one detector
class. What survives: the hardened `error_certain_regression`
primitive (unit- and identity-pinned, for Phase 3), admission
telemetry, and the measurement probes.
Registry: `docs/rfcs.md` RFC-065.

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

- UG pairing — *(as landed; this bullet originally described Phase 0's
  verbatim reuse of `belt_flow::build_ug_pairs`, direction-only)*: since
  Phase 1 slice 1 the CANONICAL pairing lives in `connectivity` itself —
  nearest same-direction, **same-name** (game rule U5) exit ahead at
  dist > 1, bucketed O((I+O)·log O) — and `belt_flow`/`belt_structural`/
  `check_underground_belt_pairs` all delegate to it. The Phase 0 fidelity
  gap (no name filter) is closed; the history lives in the decision log
  (2026-08-04 Phase 1 entry).
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
- **2026-08-05 — Bot review round 3 (of the round-2 SHA) triaged: no
  blocker; the DI-fusion structural gap closed, the two committed-corpus
  holes plugged.** The round independently re-derived the remap math and
  swept every y-moving transform ("no missed transform") — converging.
  Disposition: (minor/moderate, DI fusion vs RI-1) structurally right
  even though likely unreachable today (a coupling requires the
  producer's ENTIRE output, so a same-recipe standalone row alongside a
  fused cell needs partition-sibling geometry at minimum): containment
  now EXEMPTS machines stamped `di-cell:` (artifact ground truth from
  `di_cell.rs`'s segment stamping — no reachability argument needed);
  fused machines still count for band population. Pinned by a synthetic
  fixture with a fused producer inside the consumer band while a
  standalone producer band exists — exempt with the stamp, fires
  without it. The bot's requested solver-built discriminating fixture
  is not constructible on demand (entire-output coupling), recorded
  here instead. (minor, no consumer-level mixed-tier pin) added:
  `belt_detour` now pins that a cross-tier entrance/exit severs the run
  (two runs where direction-only pairing walked one). (minor,
  fold/island attribution weakening) acknowledged and already logged —
  restated plainly: on cleared-ledger transforms RI-1 is vacuous by
  construction and per-row rate attribution degrades to recipe-global;
  graph-derived attribution is the Phase 1 successor and this is the
  standing motivation for it. (informational, voider/kovarex parity
  uncommitted — "the single biggest unclosed risk") closed: both are
  now committed parity fixtures (kovarex self-loop @0.1/s am3;
  uranium-processing voider @0.05/s under `SurplusPolicy::Void`), and
  both pass — K65-1 is pinned on the exotic row kinds, not
  review-session folklore.
- **2026-08-05 — Bot review round 4 triaged: the K65-3 instrument gap
  closed by fixing the algorithm, plus two kind-fidelity corrections.**
  (major) `build_ug_pairs` was O(I×O) and the K65-3 bench had ZERO
  undergrounds — the class `undergroundify` mass-produces. Rather than
  just re-measure, the canonical pairing is now bucketed
  (`(name, direction, cross-coord)` → `BTreeSet` of along-coords):
  O((I+O)·log O), exact-equivalent by construction, held by a PERMANENT
  300-soup seeded equivalence pin against the retained naive reference
  (the Phase 1 probe's discipline, committed this time). New UG-dense
  bench: 11k entities / 5,500 pairs incl. a 500-pair single-row naive
  worst case — **2.48 ms** release; serpentine bench unchanged.
  (minor, splitter kinds) geometry-first now: `SplitterOut` only for
  aligned exits, perpendicular receivers are `Sideload` whatever the
  source (a receiver rotation always changes the edge set — the diff-
  blessing concern), and a head-on into a splitter's face is a recorded
  conflict (the one splitter-adjacent case the "mirrors exactly" claim
  missed); both pinned. `EntityDirection` gained `derive(Hash)` for the
  bucket key. (moderate ×2, census-only name-filter guard + silent
  decline-to-compact on residual false positives) both acknowledged
  standing residuals, unchanged from rounds 2–3 dispositions; the
  parity corpus at 14 fixtures incl. the exotic row kinds is the
  operative mitigation, and a mixed-tier structural guard remains
  Phase 1 backlog. (minor, DI-stamp trust) accepted with the sharper
  observation recorded: a displaced fused machine is invisible to RI-1
  by exemption but NOT to `validate()` — the dispatched inserter
  direction/chain checks catch a severed cell; graph-derived
  attribution subsumes this properly. (minor, orphan-output overlap
  delta + deliberately fragile fixtures) already recorded 2026-08-04/05.
  (nit, status header) fixed.
- **2026-08-05 — Bot review round 5 triaged: two structural immunity
  facts recorded, scope phrasing fixed, one count invariant added; the
  recurring majors are converged positions.** (major, name filter
  reaches parsed blueprints) defused by verification: neither
  `analysis.rs` nor the mining CLI dispatches `validate()` on imported
  blueprints, and the standing game-truth disposition applies wherever
  the walkers do run — the old direction-only pairing modeled flow the
  game does not perform, so any verdict it produced on mixed-tier runs
  was the wrong verdict. (major, RI-3 "contradicts a pinned tolerant
  contract") defused by reading the pin: `count_disconnected_ignores_
  out_of_range_and_non_pole_endpoints` pins that the CONSUMER neither
  panics nor mis-counts on junk endpoints — a defensive unit contract,
  not a claim that full validation stays silent on a corrupt stored
  graph. Tolerant consumers + loud integrity is defense in depth;
  cross-referenced in the RI-3 comment. (major, dispatched RI-1 thin
  coverage) the recurring top concern, now with the strongest
  structural answer on record: RI-1 can only fire where
  `effective_rows` exists, and ONLY the engine's bus pipeline writes
  it — parsed blueprints, hand-built layouts, and Spaghetti-style
  artifacts carry no ledger and are immune by construction. The
  check's exposure is exactly the engine's own output, which is what
  the 14-fixture corpus samples. (minor, "guards automatically"
  overstatement) fixed — dispatch comment and module doc now scope the
  claim: remap-path transforms guarded; cleared-ledger paths vacuous by
  design pending graph attribution. (minor, no transform registry)
  answered structurally: the dispatch IS the registry-free enforcement —
  any future y-moving transform that forgets the ledger fails its own
  tests' validate() calls; `apply_island_placement` needed review
  archaeology only because it predated the dispatch. (minor, presence-
  only floor) hardened: hand-edge kinds must now count EXACTLY the
  inserter population per fixture — over-emission fails. (minor, fold
  attribution trade in PR body + nit, stale PR title/body) fixed at the
  source: PR #574's title and body updated to describe Phases 0 + 1
  slice 1, the dispatch, and the behavior trade-offs explicitly.
- **2026-08-05 — Bot review round 6 triaged: the 3/3 DI-predicate
  finding fixed (a genuine self-inflicted parallel derivation), and the
  recurring name-filter major closed with its terminal answer.**
  (minor 3/3, `di-row:` exemption gap) verified and fixed: the
  canonical `validate::is_di_cell_entity` covers BOTH `di-cell:`
  (stacked, Phase 1) and `di-row:` (horizontal, Phase 2) stamps; RI-1's
  hand-rolled `di-cell:` prefix check missed the row form — and
  re-derived a predicate that already existed, this RFC's own smell.
  Now delegates to the canonical predicate; pin extended with the
  `di-row:` variant. (major 2/3, name-filter tightening "silent") the
  TERMINAL disposition, sharper than the game-truth argument alone: the
  scenario is not silent. A mixed-tier interleaving that direction-only
  pairing would have "paired" produces unpaired entrance/exit ERRORS
  from `check_underground_belt_pairs` (name-filtered since long before
  this RFC), so any input relying on cross-tier spans fails loudly at
  check #19 — the lane walkers' quieter severed treatment is backstopped
  by a hard error on the same geometry. There is no silent verdict
  change to guard against. (minor 1/3, zero-machine sibling RowSpan)
  speculative — no known path places a zero-machine row; adding
  tolerance would blunt real detection; revisit only with a repro.
  (minor 1/3, single-fixture contingency of the compact pin) standing
  deliberate design (hard-fail contract), logged 2026-08-05 round 2.
  (nit, RI-3 duplicate issues on degenerate self-loop wires) fixed —
  one issue per bad endpoint reference. (nit, reviewer lacks a Rust
  toolchain) environmental; local + CI runs are the gate.
- **2026-08-05 — Bot review round 7: convergence.** "No blocker or
  major correctness bug after extensive scrutiny," with an explicit
  verified-correct list (pairing equivalence re-derived, both remaps,
  admission loops, RI-1 calibration, anomaly rules, wire recomputes).
  Residual minors: (a) name-filter warning-set changes on already-RED
  layouts have no consumer pin — DECLINED with rationale: warning sets
  on error-bearing layouts are not a stable contract anywhere in this
  codebase (the admission gates count Errors; `validator-reporting.md`'s
  discriminating-power doctrine is about green layouts and error
  classes), and the loud backstop at check #19 covers the load-bearing
  path. (b) the `remap_y(y_end)` exactness assumption (band's last row
  occupied — guaranteed by `placer.rs` band construction) is now stated
  at the remap site with its failure signature (row stripping stalls
  loudly rather than mis-attributing). (c) the "straddling band merges
  two machine groups" scenario is REFUTED by entry-wise reasoning: the
  remap maps each ledger entry independently and monotonically — a cut
  through an in-band empty row shrinks that one band; a cut through the
  gap between two same-recipe bands makes them ADJACENT ([0,5)+[6,11) →
  [0,5)+[5,10)), never merged, and `resolve_row_spec_banded`'s
  half-open windows keep adjacent bands distinct. No ledger entries are
  ever merged on the remap path. (d) future-transform-defeats-compaction
  flagged "for completeness" — the documented registry-free-enforcement
  trade; a rejection-cause trace event for admission loops goes on the
  Phase 1 backlog.
- **2026-08-05 — Bot review round 8 (the recovered re-run of the
  degraded round) triaged: one guard added, one asymmetry declined with
  rationale, one finding refuted, one genuinely sharp observation
  recorded.** (2/3, unenforced band-occupancy assumption) taken beyond
  round 7's comment: a `debug_assert` at the strip remap now trips on
  any band whose final row is unoccupied — the padded-band failure mode
  is checkable in every debug/test run instead of latent. (1/3,
  population-vs-containment asymmetry) DECLINED: population's job is
  detecting ghost bands (no machine presence at all); a lower-edge
  straddler already produces its containment finding, and tightening
  population to full footprint would emit TWO findings for one drifted
  machine — the double-report shape, not a fix for masking. Nothing is
  masked: every drifted machine reports; the band's misdescription IS
  those findings. (1/3, Manhattan-distance comment "missing") REFUTED —
  the exact comment exists at the cited site ("Pairs are axis-aligned,
  so the axis delta is the pair distance the old loop tracked"). (1/3,
  fold admission across attribution regimes) the round's sharpest
  observation, recorded: `accept_if_no_worse` on a fold compares a
  banded-attribution input against an empty-ledger candidate — two
  different rate-attribution regimes. Not a regression (pre-fix
  candidates carried STALE bands, a third and worse regime) and
  subsumed by the documented clear trade, but named here because
  graph-derived attribution must close it symmetrically. (1/3,
  overlap-ordering question on orphan outputs) answered: dispatch is
  unconditionally parallel — `check_entity_overlaps` always runs; no
  ordering to confirm. Remaining echoes (name filter, RI-3 exposure
  breadth, dead-code IR pending per-check migration, count-not-identity
  floor, DI no-band latency, red-suite merge gating) all carry standing
  dispositions from rounds 2–7; the red-suite item stays user-owned.
- **2026-08-05 — Bot review round 9 triaged: the DI exemption narrowed
  to producers (2/2 — correct and taken), the debug_assert given a
  release-surviving voice, one sibling-path claim refuted by case
  analysis, one structural bound added.** (2/2, consumer over-exemption)
  right: only fused PRODUCERS legitimately sit in foreign bands; fused
  consumers live in their own band and now keep containment coverage.
  Role resolution follows the stamps as actually written — stacked
  cells suffix `:producer`/`:consumer`; `di-row:` stamps BOTH machine
  roles with the plain seg (verified `di_cell.rs` — `seg.clone()` for
  both), so there the producer is the machine whose recipe differs from
  the seg's trailing consumer-recipe component. Pinned both stamp forms,
  exempt and displaced-consumer directions; the DI parity fixture
  passing with consumers re-checked confirms corpus safety. (2/2,
  debug_assert compiled out of release) taken: the occupancy violation
  now ALSO pushes an artifact-carried warning — the release-surviving
  signal, `ReactivePassNotConverged` precedent. (1/2, "same tripwire
  missing on collapse_horizontal_cut") REFUTED by case analysis: the
  cut path's decrement is padding-robust — it removes exactly one known
  row, so a band with trailing padding either shrinks its padding
  (content intact) or is untouched; over-shrink requires the strip
  path's counted-removal semantics, which is where the guard lives. The
  premise is NOT identical between the siblings. (1/2, BeltFlow-family
  over-emission unbounded) taken structurally: the parity helper now
  enforces the per-source cap — at most one outgoing surface-flow edge
  per entity, two for splitters — so phantom edges fail without
  re-deriving geometry. (1/2, graph not yet load-bearing in production)
  standing Phase-1-remainder disposition; stated plainly in the module
  doc since round 5.
- **2026-08-05 — Bot review round 10 triaged: the guard-inversion
  moderate FIXED (a real catch on round-8/9's own hardening), and the
  mixed-tier thread closed for good on game-rule grounds.** (moderate,
  inverted boundary arms) right: the occupancy guard short-circuited
  `y_end > layout.height` to "occupied", silently blessing exactly the
  band shape whose remap takes the off-band fallback arm. The boundary
  arms now resolve to NOT-occupied — out-of-range ends warn. Its
  annotate-don't-correct design is kept deliberately: a strip pass that
  "corrected" a violating band would be inventing geometry; the
  admission loops are the enforcement. (minor, straddle-strictness
  coupling) standing harm-calibration disposition (rounds 2/8): a
  future transform producing top-in-band straddles is drifting off the
  construction invariant, and flagging that drift is the check's job —
  the band is consumed as a window, not only as a spec key. (minor,
  mixed-tier structural guard) CLOSED, terminally, on game-rule
  grounds rather than census grounds: mechanics rule **B12 makes
  mixed-tier same-axis interleaving LEGAL Factorio** — belt weaving
  crosses lines by pairing each tier past the other. The name filter is
  therefore not merely corpus-safe; it is what models legal weaving
  CORRECTLY (direction-only pairing mis-paired across tiers on every
  weave), and a structural guard flagging interleavings would
  false-positive on legitimate geometry. Cross-tier arrangements that
  do NOT self-pair within reach already hard-error at check #19. The
  "Phase 1 backlog" guard item is retired in favor of this record.
- **2026-08-05 — CORRECTION: the "two pre-existing failures on main"
  were never a real red — they were this session's LOCAL zone-cache
  divergence.** The CI `rust` job passed on `b1cda3b` (full suite, ci
  nextest profile — no test skipped), which prompted re-examination:
  CI pins `SPAGHETTIO_ZONE_CACHE_PATH` to the committed
  `sat-zones-ci.bin` snapshot precisely so results equal a warm local
  run "by construction"; this session's container ran the suite with
  its own divergent cache, so SAT junction zones solved differently,
  layout geometry shifted, and the calibrated warning pins of
  `tier4_advanced_circuit_from_ore_am2` / `partition_strategy_
  scoreboard` mis-compared. Verified: both tests PASS locally with the
  pin applied (90 s). What survives from the earlier entries: the
  stash A/B remains valid evidence this RFC's diff did not cause the
  local failures (identical either way); what is retracted: "almost
  certainly fallout of the #573-adjacent reverts", "main is red",
  "left for the owning session" — main was green throughout, and no
  re-bless is needed. Lesson recorded: on this repo, a full-suite run
  without the zone-cache pin is NOT a valid oracle for the
  ceiling-gated pinned tests; the ci.yml comment said so all along.
- **2026-08-05 — Bot review round 11 triaged: the guard saga ENDS in a
  falsification of rounds 7–10 (mine included), and the corpus-coverage
  major is answered with CI's own green run.** (minor 3/3, "the stated
  failure mode is physically impossible") CORRECT, verified by fresh
  derivation: `remap_y` is a pure below-count applied to entities and
  band bounds alike, and any kept row `r` in a band forces
  `remap(y_end) ≥ remap(r)+1` — containment is exact for leading,
  interior, and trailing padding without any occupancy assumption. The
  entire guard apparatus (round 7 comment → round 8 debug_assert →
  round 9 release warning → round 10 boundary-arm fix) defended a
  phantom, and worse, its assert would have PANICKED on a legitimate
  padded band the remap handles correctly. All of it removed; the
  proof sketch now lives at the remap site. Process lesson recorded:
  I accepted round 7's over-shrink premise without re-deriving it and
  then hardened the guard three times — reviewer claims about MY code
  get the same adversarial probe as my claims get from them. (major
  3/3, "K65-1 asserted green, needs a toolchain run over the full
  corpus") ANSWERED with evidence: the CI `rust` job runs the ENTIRE
  core suite under the ci nextest profile with check #40 dispatched,
  and it is GREEN on this head — that is precisely the independent
  full-corpus verification requested, performed by CI on every push.
  (minor 2/3, fold attribution change belongs in the PR body) taken —
  body updated to state the `?fold=1` output-quality change explicitly.
  (minor 1/3, graph not dispatched) standing Phase-1-remainder
  disposition. (nits) name-filter warning-sets closed round 10 (B12);
  detection-test ordering fragility noted, deterministic on the pinned
  fixtures.
- **2026-08-05 — Bot review round 12 triaged: five micro-fixes taken,
  the DI blind spot bounded precisely, the rest standing.** Taken:
  the parity belt-floor now keys on the surface-flow FAMILY (a
  hypothetical all-turns fixture is legal geometry); the equivalence
  soups straddle negative coordinates (the signing math was only
  positive-quadrant-tested); the foreign-band message says "top row
  inside" (top-y is what resolution keys on); the naive-reference doc
  says pre-BUCKETING (it pins Phase 1's name-filtered semantics, not
  the retired direction-only variant); the "collected once" comment is
  narrowed to the population direction. Bounded (2/3, DI displaced-
  producer blind spot): the residual hole is a fused producer drifting
  WITHIN its own footprint span (≤2 tiles before its coupled inserter
  hands land off-footprint and the dispatched inserter checks fire);
  rigid whole-cell displacement is caught via the consumer (checked
  since round 9); and any systematic transform bug hitting cells also
  hits non-exempt rows. Real, small, and exactly what graph-derived
  cell contracts (Phase 1 remainder) close. Standing: the name-filter
  item (closed rounds 6/10 — note round 12's sweep of
  `belt_structural` into the changed-consumer list is wrong, its copy
  was always name-filtered); fold/island vacuous-RI-1 (documented
  trade, in the PR body since round 11); the double-finding design
  (round 8 rationale: two distinct positioned statements); the
  admission-outcome pin ask (the hard-fail fixture IS the forward
  regression pin; corpus growth welcome but not gating).
- **2026-08-05 — Phase 2 slice 2a (post-merge, fresh branch): the
  error-certain pre-filter primitive lands; the cut-path measurement
  says the money is on the fold path.** Built:
  `connectivity::error_certain_regression` — the sound reject-fast
  detector over index-stable diffs (net span loss per entrance, net
  hand loss per inserter with retargets deliberately falling through,
  added same-carries head-ons), unit-pinned per class; wired into both
  cut loops via `cut_admission` with base-graph caching and admission
  telemetry (`CutAdmissionStats`), toggleable through
  `compact_validated_geometry_with_stats`. Soundness gate K65-5: cut
  outcomes must be BYTE-IDENTICAL filter-on vs filter-off (pinned;
  holds). MEASUREMENT: the cut path has almost no validate() volume to
  save — the cut constructors refuse most bad geometry structurally
  before validation (EC@2: six validated candidates, zero
  filter-catchable; EC@20: one). The filter engagement number is
  reported, not asserted, and the pin is honest about why. CONSEQUENCE,
  recorded as the next slice's scoping: the validate() bottleneck that
  motivated Phase 2 lives in `search_snake_fold` (a comb of candidates
  × full validate each — the reason `FOLD_SEARCH_ENTITY_THRESHOLD`
  exists), and fold candidates are index-UNSTABLE (fold_snake rebuilds
  and reorders its entity list), so the fold-side pre-filter needs an
  old→new identity map exported from the fold transform. Kill criterion
  for that slice: if the mapped filter cannot reject-fast ≥30% of fold
  candidates that validation rejects on the fold corpus (chain-mil5ore
  + the two admissible row-bus fixtures), the mapping machinery is not
  worth its complexity — measure on the spike before wiring.
- **2026-08-05 — Phase 2 slice 2b: fold-side pre-filter KILLED on the
  pre-registered criterion; the measurement instrumentation is the
  slice's deliverable.** Design first: the identity map slice 2a scoped
  turned out to be unnecessary — the fold search's `profile()` returns
  `None` (silent discard) for any Error-carrying candidate, so a
  candidate-only anomaly scan (`scan_graph_anomalies`: unpaired UG
  halves, unbound hands, same-carries head-ons — all Error-certain)
  is sound with no base comparison and no index mapping at all. That
  version was built, wired toggleably, and held the K65-5 byte-identity
  pin on the admissible fold fixture (AC@5-from-plates). MEASUREMENT
  (via the new `CutAdmissionStats::error_discards` counter — of the
  validates run, how many the validator Error-rejected, i.e. the only
  volume any sound Error-certain filter could ever reject-fast):
  gear15-ore 0 discards / 0 validates (130 structural refusals),
  ec10-ore 0/0 (30 refusals), ac5-plates 0/62 (all 62 pass;
  regression_rejects=0), chain-mil5ore 1/151 (119 warning-regression
  rejects). Of the corpus's 120 validation-rejected candidates, ≤1
  (0.8%) was Error-class — the ≥30% criterion is tripped by a factor
  of ~40. WHY: `fold_snake`'s `FoldRefusal` machinery structurally
  refuses Error-certain geometry before a candidate exists (that was
  RFC-057's design), so validation volume is spent on candidates that
  pass or regress on warnings — untouchable by a sound Error-certain
  filter. DISPOSITION per the stop rule: the fold-side filter is
  REMOVED, not shipped default-off (62–151 dead graph derivations per
  search for zero savings); what ships is the admission telemetry
  (`search_snake_fold_with_stats`, `error_discards` on both paths) and
  the two `#[ignore]` measurement probes
  (`phase2b_fold_prefilter_measurement` in `connectivity_parity.rs`,
  `phase2b_fold_admission_volume_chain_mil5ore` in
  `cell_composition.rs`) that keep the negative result checkable. The
  cut-side filter (slice 2a) stays as committed: byte-identity pinned
  (now also counter-form: rejects must map 1:1 onto baseline Error
  discards), negligible volume (6 validates on its pin fixture), and
  it is the `error_certain_regression` primitive's only production
  call site — the primitive's real customers are Phase 3 transforms,
  which will NOT have per-transform refusal machinery. CONSEQUENCE for
  the backlog: "fold identity map" is moot (dropped); the fold-search
  cost lever is not Error rejection but the 119-strong
  warning-regression volume — any future slice there must predict
  *warning* profiles from the graph, a different (unscoped) premise.
- **2026-08-05 — Local adversarial review of both Phase 2 slices:
  slice 2a's soundness claim FALSIFIED; detector hardened, cut-side
  filter demoted to default-off test machinery.** The reviewer built a
  working counterexample (finding 1, blocker): `collapse_vertical_cut`
  calls `normalize_adjacent_undergrounds`, which rewrites a
  cut-adjacent UG pair to surface belts IN PLACE — entity count
  unchanged, so the count-equality guard engaged the filter, the diff
  saw the entrance's span vanish, and the filter rejected a candidate
  `validate()` admits with zero Errors (probe: filter-on stuck at
  width 5 / 5 entities vs width 1 / 1 entity off; a dist-2 UG pair
  with a decoy belt anchoring the gap column). Not exotic: iterative
  cuts shorten any progressively-cuttable span gap to the dist-2 →
  adjacent step. Finding 2 (concern): the "lost a hand binding" class
  rationale was factually wrong — no validator check errors an unbound
  hand per se (`check_inserter_chains` errors machines without
  inserters; `check_inserter_direction` errors only when neither hand
  touches a machine; coverage/input-rate backstops tolerate redundant
  inserters or emit Warning) — so a redundant inserter losing its
  belt-side pickup is validate-admissible. Finding 3 (concern): the
  K65-5 byte-identity pin was VACUOUS — its fixture never engaged the
  reject path (0 rejects / 6 validates), so it could not have caught
  finding 1; weak evidence dressed as a gate, the recurring
  check-went-quiet shape from `docs/validator-reporting.md`. What
  survived attack, per the reviewer's own re-runs: the 2b telemetry
  refactor (line-level behavior-identical), the `error_discards`
  counter semantics (validate() Errs iff an Error-severity issue
  exists), all four corpus measurements (reproduced exactly), and the
  fold-side kill-criterion arithmetic and disposition. FIXES, same
  session: span-loss class narrowed (fires only if the node is STILL a
  UG entrance in `after` — sound because derive and check #19 share
  the canonical pairing since Phase 1); hand-binding class REMOVED
  (negative unit pins now guard both unsound classes against
  re-introduction); cut-side filter flipped to default OFF everywhere
  (`compact_validated_columns/rows/geometry`) — its measured benefit
  was already zero, and post-review the burden is on evidence of
  benefit, not absence of divergence; the reviewer's counterexample
  ported as `phase2_prefilter_identity_on_ug_normalizing_cut`, a pin
  that engages the filter and FAILS on the unguarded class rather than
  passing vacuously. Production cut behavior is back to pre-2a
  (pure-validate admission). Nits absorbed: status-line share
  corrected to 0.83% (1/120); ac5-plates note corrected to "all 62
  pass" (regression_rejects=0); the 2a entry's "EC@20: one validate"
  stands as a session observation but has no committed instrument —
  treat the four probe-backed numbers as the reproducible record.
- **2026-08-05 — PR #579 bot round 1 (second-opinion, union ×2): three
  fixes pushed, one refuted with a committed assertion, one answered
  in-thread.** (1) Major, and the round's real catch: the
  "error-certain" contract rested on prose — the unit pins proved the
  detector FIRES but never ran `validate()` to prove the classes are
  Error-certain. Fixed with
  `error_certain_classes_are_validator_errors`: each class's
  after-layout must carry the specific validator Error category
  (`underground-belt` for span loss, `belt-junction` for head-ons)
  that the base lacks — an unsound class is now a one-line failure,
  not a prose dispute. (2) Minor, accepted: the head-on class lacked a
  `ConflictKind::HeadOn` guard (correct today only because HeadOn is
  the sole variant; would silently broaden with a new kind) — guard
  added. (3) Minor, accepted: pin-vacuousness had a second face — the
  UG pin's `off.width` guard proved compaction happened, not that the
  filter ENGAGED. Added `CutAdmissionStats::prefilter_evals`
  (count-equality branch taken) and the UG pin now asserts it nonzero.
  (4) Minor, REFUTED: the claim that the retarget sub-case's drop tile
  goes empty is wrong on geometry — `machine(4,3)` is a 3×3
  assembling-machine-2 covering the (4,4) drop tile; settled by
  asserting the `InserterDrop 5→6` edge exists in the retarget
  fixture (the dead `name` assignment the bot also flagged was real
  and removed). (5) The "no fixture drives `prefilter_rejects > 0`"
  gap is answered in-thread, not fixed: through the production cut
  loop no such fixture is REACHABLE — cuts only shorten spans, occupied
  columns aren't cuttable, and adjacent pairs normalize in place;
  that unreachability is the Phase 2 measurement (zero catchable
  volume), i.e. the reason the filter is off. Primitive-level
  soundness is now held by the direct contract pin instead, which is
  strictly stronger than a wired-path reject would be.
