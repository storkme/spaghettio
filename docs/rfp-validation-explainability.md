# RFP: Validation explainability — from warning soup to visible causes

## Summary

The validator finds real problems, but the web UI surfaces them as
atomized one-line symptoms (`[warning] input-rate-delivery: needs
3.0/s, delivers 1.4/s` × 35), and the *causal* knowledge the engine had
at stamp time — which side was under-provisioned, why, what blocked the
slot — is computed and then thrown away. A human eyeballing the layout
has to re-derive the mechanism by hand (as happened 2026-07-14: the
user traced EC@35s starvation to inserter slots blocked by the
template's pole-gap reservation — a fact the inserter ladder knew when
it capped the side). This RFP adds four composable pieces: **(1) a
starvation heatmap** overlay, **(2) stamp-time causal attribution**
carried on validation issues, **(3) click-to-explain** entity
highlighting, and **(4) a cause rollup** in the sidebar. Pure
metadata + UI — zero layout-output movement by construction.

## Motivation

Reproducible today: `#/l/ecl/35/am2/ior,coo/tbr` (the
`stress_electronic_circuit_35s_from_ore` cell). The shipped layout
carries 35 `input-rate-delivery` warnings. Finding out *why* took a
human staring at tiles: machines needing 2× long-handed inserters have
one, because the dense templates reserve dx=2 for a power pole
(`templates.rs` "leaving dx=2 free for `place_poles`") and the
long-handed count-ladder caps at the slots left over. The engine knew
this at stamp time (`inserter_ladder` computes planned vs fitted per
side); the validator later rediscovers only the downstream symptom; the
UI shows one hover line per machine (`validationOverlay.ts:44`).

Every future eyeball session pays this cost. The verification protocol
(CLAUDE.md step 2, "look at the layout with your eyes") and any future
in-game ground-truthing (inserter-sizing KC5) get dramatically cheaper
if the layout explains itself.

Related but explicitly **out of scope**: actually *fixing* the
pole-vs-inserter priority inversion (a human gives inserters first
claim on face tiles and lets poles hunt; the engine reserves the pole
gap up front). Parked by the user 2026-07-14 ("maybe we'll come back to
this") — it belongs to the face-allocation north-star. This RFP's job
is to make that future RFP's evidence gathering trivial.

## Design

### D1. Structured payloads on `ValidationIssue`

`ValidationIssue` (validate/mod.rs:66) gains an optional structured
field:

```rust
pub struct IssueDetail {
    /// Machine-readable numbers the message already states in prose.
    pub delivered: Option<f64>,
    pub needed: Option<f64>,
    /// Stamp-time cause, when a provision record matches (D2).
    pub cause: Option<String>,          // human-readable, one line
    pub cause_kind: Option<String>,     // stable enum-ish slug
    /// Entities that constitute the explanation (D3 highlights these).
    pub related: Vec<(i32, i32)>,
}
pub struct ValidationIssue { ..., pub detail: Option<IssueDetail> }
```

Serialization crosses the WASM boundary with the existing derive
plumbing (same pattern as prior field additions). Checks that already
compute delivered/needed (`input-rate-delivery`,
`inserter-item-throughput`, lane-throughput) populate the numbers
instead of only formatting them into prose.

### D2. `side_provisions` ledger (the attribution source)

Following the `effective_rows` / `voided_streams` pattern: a pure
metadata ledger on `LayoutResult`, written at stamp time, never read by
layout logic.

```rust
pub struct SideProvision {
    pub machine_x: i32, pub machine_y: i32,
    pub face: Face,                    // N/E/S/W
    pub item: String,
    pub planned_rate: f64,             // what the ladder was asked for
    pub fitted_rate: f64,              // what the placed inserters give
    pub inserters: Vec<(i32, i32)>,
    pub limit: ProvisionLimit,         // why fitted < planned, if it is
}
pub enum ProvisionLimit {
    None,                              // fully provisioned
    SlotReservedForPole { tile: (i32, i32) },
    SlotGeometry,                      // no free tile on this face
    TierCap,                           // max_inserter_tier ceiling
}
```

Populated where the ladder/templates already know the answer
(`inserter_ladder::size_side` call sites in `templates.rs` /
`placer.rs`). **Attribution is a join, not an inference**: at validate
time, an `input-rate-delivery` or `inserter-item-throughput` issue
looks up the provision record for the same machine/face/item and copies
its `limit` into `IssueDetail.cause`. If no record matches, `cause`
stays `None` — the UI says "unattributed", never guesses.

### D3. Web UI

- **Heatmap toggle** (Phase 1): color machine sprites by
  `delivered/needed` (from D1 numbers; red → starved, green → fed).
  Rides the existing overlay-toggle pattern
  (`validationOverlay`/`regionOverlay` siblings).
- **Click-to-explain** (Phase 3): clicking an issue marker highlights
  `detail.related` (the side's inserters, the blocking pole tile, the
  feeding belt segment) via the existing selection/segment-highlight
  machinery, and shows `cause` in the hover/inspector text.
- **Cause rollup** (Phase 3): sidebar panel grouping issues by
  `(category, cause_kind)` with counts — "input-rate-delivery: 28 ×
  slot-reserved-for-pole, 7 × lane under-provision" — replacing
  warning-count soup with a diagnosis.

### Trade-offs / rejected

- **Post-hoc inference in the validator** (guess causes from geometry
  at validate time): rejected — it re-derives what stamp time already
  knew and will be wrong at the margins; the whole point is carrying
  facts forward.
- **Message-string parsing in TS** for the heatmap: rejected —
  structured fields are ~the same diff size and don't rot.
- **Snapshot format**: `.fls` serializes `LayoutResult`, so the ledger
  appears in snapshots for free (reader tolerates missing fields on old
  snapshots — same as `effective_rows`).

## Kill criteria

1. **Zero layout movement, absolute**: if adding the ledger/detail
   fields moves ANY golden hash or e2e fixture byte
   (`SPAGHETTIO_STRESS_GOLDEN` before/after), the "pure metadata"
   premise is violated — stop and rethink; do not re-bless.
2. **Attribution coverage on the anchor case**: on
   `stress_electronic_circuit_35s_from_ore`, if fewer than ~80% of the
   35 input-rate-delivery warnings get a non-None `cause` from the
   ledger join, the `ProvisionLimit` vocabulary is wrong (the causes
   live elsewhere) — stop before building UI on top of it.
3. **No inference creep**: if making criterion 2 pass requires the
   validator to *compute* causes (anything beyond a key join on
   machine/face/item), D2's design is wrong — that's the post-hoc
   guessing this RFP exists to avoid.
4. **Runtime**: if validate+layout regresses >5% on the e2e corpus
   from ledger collection, trim the ledger (record only capped sides)
   or kill.

## Verification plan

- Full suite + `SPAGHETTIO_STRESS_GOLDEN` before/after (criterion 1).
- Unit tests: ladder call sites emit provisions (planned vs fitted vs
  limit) for a capped side and a clean side; issue join copies the
  right cause; no-match leaves `cause: None`.
- Anchor case: EC@35s — count attributed vs unattributed ird warnings
  (criterion 2); spot-check 3 attributions against the entity map (the
  pole tile named in `SlotReservedForPole` must actually hold a pole).
- UI: per `feedback_user_validates_ui` — land the heatmap/click/rollup
  and let the user eyeball on the EC@35s URL; no screenshot iteration.

## Phasing

1. **Phase 1 — numbers + heatmap**: `IssueDetail` with
   delivered/needed on the rate checks; heatmap toggle. Smallest
   landable visible win.
2. **Phase 2 — provisions + attribution**: `side_provisions` ledger,
   the validate-time join, `cause`/`cause_kind`/`related` populated.
   Criteria 2+3 gate here.
3. **Phase 3 — explain UI**: click-to-explain + cause rollup panel.

## Decision log

- *2026-07-14 — drafted after the user's EC@35s eyeball session traced
  input-rate-delivery starvation to pole-blocked inserter slots by
  hand; user: tooling "sounds worthwhile", pole-priority fix itself
  parked ("maybe we'll come back to this").*
