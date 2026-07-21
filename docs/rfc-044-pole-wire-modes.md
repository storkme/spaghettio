# RFC-044: Pole wire modes — dense mesh vs deterministic minimal tree

Status: ACCEPTED v2 (2026-07-21) — adversarial review round 1
complete; two must-fixes and two wording fixes folded in (decision
log). Ready to implement.

## Summary

Add a user-facing **pole wiring mode**: `dense` (today's behavior —
every in-reach pole pair gets a copper wire, maximally robust) or
`tree` (a deterministic minimum spanning tree per connected component —
fewest wires that keep the network connected, visually clean,
`|poles|−1` edges instead of a mesh). Alongside the toggle, restructure
the wire-graph contract from "N call sites re-derive and must agree"
to **"computed once, stored on the layout, all consumers read the
stored graph"** — which retires the divergence-bug class the
`power_wires` module's docs currently police by convention. Default is
`dense`; Normal-and-default output stays byte-identical.

## Motivation

Aesthetic/user-choice, not correctness: dense wiring on a legendary
layout (sparse poles, 17-tile spans, per `rfc-043-pole-band-thinning`)
renders as a cross-hatched mesh; players routinely prefer minimal
wiring. The trade-off is real and belongs to the user: in a tree,
deconstructing any single pole splits the network, where the mesh is
redundant. Secondary wins: smaller `wires` arrays in exported strings,
and the structural fix below.

Honest note on in-game semantics: tree-ness is not preserved by
in-game *repair* — a pole rebuilt by bots auto-connects to everything
in reach, locally re-meshing. The mode governs what the blueprint
pastes, nothing more.

## Current state (the four consumers)

`power_wires::compute_pole_wires(&entities)` — per-entity quality
reach (`common::pole_wire_reach`), min-of-both pairing, every in-reach
pair, normalized `(a, b)` index pairs with `a < b` — is today invoked
independently by:

1. `bus::layout::layout_pass` (`layout.rs:1065`) — populates
   `LayoutResult.power_wires` (after the quality stamp pass; ordering
   is load-bearing) for the web overlay.
2. `blueprint::export` (`blueprint.rs:138-150`) — **re-derives** and
   encodes the blueprint-level `wires` array
   (`[a+1, POLE_COPPER, b+1, POLE_COPPER]`).
3. `validate::power::check_pole_network_connectivity`
   (`power.rs:40-41`) — **re-derives** and checks the emitted-artifact
   graph via `count_disconnected_poles`.
4. `wasm-bindings::improve_region_streaming` — **recomputes** after
   its entity-reordering splice (index pairs invalidate on reorder;
   documented on the `power_wires` field, `models.rs:351-358`).

`blueprint_parser` already does the fidelity-correct thing: it
resolves an imported blueprint's actual `wires` array (copper
connector-5 edges) into `power_wires` (`blueprint_parser.rs:312,
393+`) — imported graphs are preserved verbatim, never recomputed.

A mode toggle under the current shape would need the mode threaded to
sites 2–4 and kept in agreement forever. Instead:

## Design

### 1. The stored-graph contract

- `LayoutResult.power_wires` becomes `Option<Vec<(u32, u32)>>`:
  - `Some(graph)` — authoritative; consumers use it verbatim (even
    `Some(vec![])`, e.g. a single-pole layout).
  - `None` — "never computed" (hand-built `LayoutResult`s in tests,
    corpus tools, legacy snapshots predating power-3c). Consumers that
    need a graph fall back to `compute_pole_wires(entities,
    WireMode::Dense)` — exactly today's behavior, so nothing that
    works today can silently lose its wires. The `Option` is what
    makes "unpopulated" distinguishable from "legitimately empty";
    a plain `Vec` cannot express that, and exporting an unpopulated
    layout with zero wires would paste power-dead islands.
  - Serde: `#[serde(default, skip_serializing_if = "Option::is_none")]`
    — old snapshots (field absent) → `None` → fallback ✓; power-3c-era
    snapshots (field present) → `Some` ✓.
- `LayoutResult.wire_mode: WireMode` (new field, `#[serde(default)]`,
  skipped when `Dense`): the layout records its own wiring policy, so
  consumer 4's post-splice **recompute runs in the recorded mode** —
  without this, an improve-region pass on a tree layout would silently
  re-densify it.
- Consumers 2 and 3 switch from re-deriving to
  `layout.power_wires_or_derive()` (a small accessor holding the
  fallback in ONE place). Consumer 1 computes-and-stores; consumer 4
  recomputes-and-restores in `layout.wire_mode`.
- The parser sets `Some(imported_graph)` (it already builds exactly
  this) and leaves `wire_mode` at `Dense`-default — the mode field
  describes how WE generate, not a claim about imports; imported
  graphs are `Some` and therefore never regenerated in any reachable
  path. (Precisely: the only regeneration site is the improve-region
  recompute, and parsed layouts carry `regions: vec![]`, so
  `optimizeAllRegions` finds no crossing zones and never invokes it —
  review finding 5's reachability argument, stated here so a future
  change to parser region derivation knows to revisit this.)

Round-trip consequence, now guaranteed by construction: export a tree
layout → parse it → `power_wires` is the tree again (the parser reads
the artifact), and the validator validates that same tree. The
"artifact cannot be lied about" property strengthens from convention
to construction.

### 2. `WireMode` and the deterministic MST

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WireMode {
    #[default]
    Dense,
    Tree,
}

pub fn compute_pole_wires(entities, mode: WireMode) -> Vec<(u32, u32)>
```

`Tree` runs Kruskal over the SAME candidate edge set `Dense` produces
(in-reach pairs under per-entity quality reach, min-of-both — the
physics is mode-independent), with edges sorted by the **physical**
key (v2, review finding 6):

```
(squared_center_distance, min_endpoint_pos, max_endpoint_pos)
// endpoint_pos = the pole's (x, y) anchor; endpoints canonically
// ordered per edge; full lexicographic total order
```

- **Squared** Euclidean center distance: centers are `k + 0.5` /
  `k + 1.0` grid values, so squared distances are exact in f64 at any
  realistic layout size — no sqrt, no rounding wobble.
- The tiebreak is **position-derived, not entity-index-derived**: the
  codebase deliberately treats entity Vec order as a non-canonical
  implementation detail (`golden_hash` sorts before hashing), so a
  tree keyed on indices would silently change shape under a
  legitimate pipeline reordering. With the physical key, the selected
  wire SET is a unique function of the physical layout; tie or no
  tie, MST uniqueness follows from the total order. Determinism is a
  project-level contract (byte-identical output: URL state, `.fls`
  snapshots, goldens), and an order-permutation fixture (kill 3)
  proves the invariance rather than assuming it.
- Union-find: small local implementation. (v1 floated extracting the
  one inline in `repair_pole_connectivity`; dropped — that function is
  placement-time code and kill 5 fences placement off. A shared
  union-find is a possible follow-up chore, not part of this RFC.)
- Output re-sorted by `(a, b)` — same normalization and ordering
  convention as `Dense`, so downstream consumers see one format.
- Disconnected pole fields (possible on degraded layouts) yield a
  spanning **forest** — the validator still counts and warns about
  disconnected components exactly as it does today; `Tree` never
  papers over a genuine disconnection.

### 3. Plumbing (the established ladder)

`LayoutOptions.wire_mode: WireMode` (default `Dense`) → wasm `layout`
/ `layout_traced` / `layout_streaming` gain a trailing
`wire_mode: Option<String>` (unknown/absent → `Dense`, the
`max_inserter_tier` pattern; trailing-optional keeps existing JS calls
valid) → `engine.ts` → `FormState.wireMode` + URL short code `w=`
(`t` = tree; absent = dense) → sidebar select ("Pole wiring: Dense
(default) / Minimal tree"). `export_blueprint` and `validate` need NO
new params — they read the stored graph. Renderer: the existing
overlay draws whatever `power_wires` holds; only the TS type changes
(`Option` → already-optional `power_wires?`).

## Kill criteria

1. **Default identity.** With `wire_mode` absent/`Dense`: full suite,
   `SPAGHETTIO_STRESS_GOLDEN=check`, and byte-equal exported
   blueprint strings on the quality differential fixtures
   (stored-graph consumption must be indistinguishable from
   re-derivation for pipeline-produced layouts — same function, same
   entities, same bytes). Any default-mode diff ⇒ the restructure
   leaked — halt.
2. **Fallback honesty (the pre-agreed escape hatch).** If switching
   export/validator to stored-wires breaks ANY existing round-trip,
   corpus, or snapshot fixture in a way the `None`-fallback does not
   cleanly absorb, abandon the stored-graph contract and fall back to
   threading the mode parameter to all four sites (the design we
   explicitly rejected as second-best — record why it won).
3. **Determinism.** `Tree` on identical input must be bit-identical
   across runs AND stable under a tie-heavy synthetic fixture
   (equidistant pole line where every adjacent pair ties). A unit test
   constructs ≥3-way ties and pins the exact edge list.
4. **Tree property.** On every fixture where `Tree` runs:
   `edges == poles − components` (components counted by the test's
   own union-find over the dense candidate set), every edge is a
   member of the dense candidate set, and
   `count_disconnected_poles(entities, tree)` equals
   `count_disconnected_poles(entities, dense)` — the validator's
   actual scalar, identical between modes by construction (v2
   wording, review finding 7).
5. **Scope fence.** No placement, thinning, or substation changes
   (`repair_pole_connectivity` explicitly included — the v1 "optional
   union-find extraction" is dropped); if the accessor/fallback
   design requires touching those, stop.
6. **No silent re-densify (v2, review finding 10).** A Tree-mode
   layout passed through `improve_region_streaming` must come back
   with `power_wires` still satisfying the tree property of kill 4 —
   the recompute at `wasm-bindings/src/lib.rs:414` must honor
   `layout.wire_mode`, and a test proves it (build a Tree layout with
   ≥1 crossing-zone region, run the improve pass, assert tree-shaped
   wires after). If the mode field cannot survive the wasm boundary
   (contradicting the `trace`/`RegionKind` precedents), that is a
   kill-2-style abandon signal for the stored-mode design.

## Verification plan

- Unit: tie-heavy determinism fixture (kill 3) PLUS the
  order-permutation fixture — the same physical poles constructed in
  two different entity orders must select the same physical wire SET
  (v2, review finding 6); tree property per component incl. a
  deliberately disconnected two-cluster fixture (kill 4);
  `None`-fallback exports wires for a hand-built layout (the existing
  `export_emits_pole_copper_wires` test covers exactly this path — it
  must keep passing unmodified); the kill-6 improve-region
  re-densify test.
- Round-trip: export `Tree` → parse → `power_wires` equals the tree
  edge set; re-export → byte-identical `wires` array.
- Differential: 45/s legendary fixture — dense edge count vs tree
  `== 29` (30 poles, one component; pin exact), both 0 power issues.
- Byte-equality harness for kill 1 on the two quality differential
  fixtures (export strings pre/post restructure at Dense).
- Browser eyeball (user): legendary census URL with `w=t` — single
  clean wire runs in the overlay.

## Sequencing

One phase; UI ships with it (no guard-rail split needed — the engine
default is inert). Registry: RFC-044, row added in this commit's
`docs/rfcs.md`, next number bumped to RFC-045.

## Decision log

- *2026-07-21 — spec'd; the stored-graph contract chosen over
  param-threading (user decision, 2026-07-21: "the cleaner structural
  fix"), with param-threading kept as kill-2's explicit fallback.*
- *2026-07-21 — adversarial review round 1. Verified sound against
  code: the Option migration (golden hashes provably never touch
  `power_wires`; web already treats the field as optional; consumer
  enumeration complete — `region_reimprove` reachable only via
  `improve_region_streaming`), the wasm round-trip (working
  precedents: `trace: Option<Vec<..>>`, `RegionKind` enums), and the
  parser/optimize non-reachability of the imported-wire clobber. Two
  must-fixes folded into v2: **tiebreak retargeted from entity index
  to physical coordinates** (entity Vec order is non-canonical by the
  project's own golden-hash convention; index-keyed ties would let a
  legitimate reorder silently reshape trees) with an
  order-permutation fixture added to kill 3; and **kill 6 added** for
  the improve-region re-densify hazard the Design section named but
  never tested. Wording fixes: kill 4 now asserts on
  `count_disconnected_poles`'s actual return; the union-find
  extraction dropped to resolve the kill-5 self-tension. Ready to
  implement.*
