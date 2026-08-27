# RFC-074: Cells at the surface — RFC-072 Phase 3, scoped

Registry: [`rfcs.md`](rfcs.md). Status: **Design → Active 2026-08-27.**
This is RFC-072's uncommitted Phase 3 ("heterogeneous composition and
the library; the web surface") scoped to what its own evidence supports:
the surface is a measured defect and ships; the fluid/mega grid path gets
its receipt or its refusal; heterogeneous composition is adjudicated by
measurement, not built; the library stays parked where K67-3 left it.

## Summary

RFC-072 Phases 0–2 made rates past the lane wall ship: the cell-chain
grid composes ec@240 as 2×12 strips and sims at plan where the native
bus panics. Nothing above the engine knows. The browser already reaches
the grid — `cell_composition: Candidate` is the engine default, the wasm
option parser maps an absent value to it, and `web/` has no rate ceiling
— so a user who types electronic-circuit at 240/s gets the composed
layout and then **cannot export it**: the composition's registry note
(a *receipt* — "SIM-VERIFIED at plan … PASS produced 240.00/s") is
pushed onto `LayoutResult.warnings`, and `web/src/ui/sidebar.ts` hides
the blueprint section whenever that list is non-empty. The renderer
draws no strip or cell outline, and the winning candidate's name reaches
the trace but no UI. This RFC (1) gives composition a typed receipt on
`LayoutResult` and a surface in the web — export restored, the receipt
shown as what it is, strips outlined; (2) closes RFC-072 residual (f)
by receipting or loudly refusing the mega/fluid grid path, and receipts
the from-plates ec@240 grid the browser actually builds; (3) adjudicates
"chains of unlike cell groups" with one number — how much a uniform K
over-provisions the chain-eligible corpus — and either closes it or
hands it to its own RFC. Kill criteria keep every part additive: no
shipped geometry moves.

## Motivation

Reproducible today (`examples/web_probe_ec240.rs`, the browser's entry
point `build_bus_layout(LayoutOptions::default())`, 2026-08-27):

| request | winner | entities | `warnings[0]` |
|---|---|---|---|
| ec@240 from ore | `cell-composed` (2×12, 4 pole bridges) | 17,148 | `cell-composed: geometry SIM-VERIFIED at plan (… PASS produced 240.00/s …)` |
| ec@240 from plates | `cell-composed` (2×12) | 4,460 | `cell-composed: geometry NOT sim-verified (hash 5c83b4199631358a)` |
| ec@150 from ore | `cell-composed` (K=12 strip) | 10,392 | `SIM-VERIFIED … PASS produced 150.00/s` |

In every row `layout.warnings.length > 0`, so
`blueprintSection.style.display = "none"` (`sidebar.ts:1164`) — the
receipt that says the layout is good is what stops the user exporting
it. The from-plates row is the request a browser user makes first
(plates are the ordinary external inputs), and it has no receipt at all.

The other two items are RFC-072's own residual list: (f) "the mega/fluid
grid path is untested by construction — no fluid-touching chain reaches
K > 12 in the corpus", and Phase 3's "chains of unlike cell groups",
for which no failing case exists today — the recon
(`~/.local/state/codex-task/p3-recon/report.md`) found the composer
bakes K-uniformity into quantization, cell cloning, corridor scope,
bypass rows, pole placement, and the registry key, and that no
heterogeneous composer exists in the tree. Building one on no measured
need is the rework shape the RFC template exists to stop.

## Design

### Unit 1 — the receipt (engine, additive)

`LayoutResult` gains

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub composition: Option<CompositionReceipt>,

pub struct CompositionReceipt {
    pub kind: String,            // "cell-chain" | "cell-grid" | "mega-cell"
    pub copies_per_strip: Vec<i32>,
    pub strips: Vec<StripRect>,  // { x, y, width, height, copies } in layout frame
    pub verification: String,    // the registry note, verbatim
    pub verified: bool,          // registry row present with a non-FAIL verdict
}
```

filled by the chain/grid composer (strip rects are known at
`append_strip_translated` time) and by the `CellComposedCandidate`
where `verification_note` is computed today. `warnings` no longer
carries the note. The note's text and the registry's mismatch
precedence are untouched (the `mismatch_note_is_fail_dominant…` tests
keep passing on `composition.verification`). Additive: entities,
blueprint bytes, registry hashes, bank fingerprints all unchanged.

### Unit 2 — the surface (web)

- `sidebar.ts`: the blueprint section hides on `warnings` only; a
  composed layout shows a composition badge — kind, `strips × copies`,
  the verdict phrase, and the full note on hover.
- Renderer: strip outlines from `composition.strips` (a layer next to
  the trace overlay's row boundaries; off when `composition` is null).
- The winner's name (`SelectionDecided.winner`, already in the trace
  the streaming path retains) shown in the validation panel header.

### Unit 3 — receipts (sim)

- **ec@240 from plates** (hash `5c83b419…`): export, sim, registry row.
  Whatever it measures is the receipt; selection is unbiased by design
  (RFC-051 kill 3) so a short measurement does not change what ships,
  it changes what the badge says.
- **The mega/fluid grid.** The smallest fluid-touching request past
  K_MAX is advanced-circuit from ore + crude at ≈48/s (cable ≈10 per
  AC → 480/s → K=13). Compose it through `compose_grid_with_capacity`:
  either it carries zero validator errors, its per-strip boundary
  records (fluid heads included, #732's direction fix in) let the
  harness rig it, and it sims with the two strips within 2 points of
  each other — then it is receipted and (f) closes — or
  `chain_eligible_at` refuses fluid-touching chains past K_MAX **by
  name** and (f) closes as "refused, not shipped". Both are closes;
  the RFC does not commit to which.

### Unit 4 — heterogeneous adjudication (measurement only)

An ignored probe over every chain-eligible corpus solve: for the K the
quantizer picks, the over-provisioning ratio
`Σ_spec ceil(count/K)·K / Σ_spec count` (machines placed over machines
needed) and the worst single spec. The decision-log entry records the
distribution. K74-3 decides. No composer is written under this RFC.

### Out of scope, deliberately

The library ("celldb-backed reuse with receipts as library rows in the
calibration bank"). RFC-067 killed its consumer under K67-3 and parked
template candidates; #619/#629 hold the state. The recon's gap list
(stable fragment + port identity, receipt fields on fragments, a
composition record, a regeneration gate) is a design of its own, and
nothing in Units 1–4 needs it. It reopens only by a decision-log
amendment on RFC-067, with a measured consumer.

## Kill criteria

- **K74-1 (additive or nothing).** If Unit 1 needs any change to
  entities, blueprint bytes, or selection — anything beyond an
  additive `#[serde(default)]` field and the note's channel — stop; the
  registry gate `cell_registry_hashes_current`, the calibration
  fingerprint probe, and the copy-count pins must be byte-for-byte
  green throughout.
- **K74-2 (the fluid grid closes, either way).** If the AC≈48 grid
  neither receipts (0 errors, strips within 2 points, sim ≥ 97.9%
  produced — the family bar) nor can be refused by name at the
  eligibility gate without touching `mega.rs` internals, Unit 3's
  fluid half stops and (f) stays open with the reason recorded.
- **K74-3 (heterogeneous pays or closes).** If the corpus median
  over-provisioning under uniform K is ≤ 5% and no fixture exceeds
  15%, "chains of unlike cell groups" closes by measurement (with the
  library). Above that, it becomes its own RFC with these numbers as
  its motivation — still not built here.
- **K74-4 (runtime).** End-to-end runtime > 2× on the existing corpus
  drops the offending unit even if the surface improves (K72-5's
  rule).
- **K74-5 (the surface is honest).** The badge must show FAIL/WARN
  verdicts and "NOT sim-verified" as prominently as PASS; if the
  from-plates ec@240 receipt comes back short and the badge would read
  as an endorsement, the badge is wrong, not the receipt.

## Verification plan

Per the CLAUDE.md layout-engine protocol. Unit 1: `cargo test` green;
registry gate + fingerprint probe unchanged (K74-1); the receipt pinned
on ec@240 (kind `cell-grid`, `copies_per_strip [12,12]`, two strip rects
separated by 32, `verified == true`) and on an unregistered hash
(`verified == false`). Unit 2: `tsc` + vitest; the six-site rule for
sidebar controls; the user eyeballs (per the standing UI rule). Unit 3:
sim receipts in `cell-sim-registry.json` with the gate config + probe
entry, as #733 did. Unit 4: the probe's table in the decision log.

## Phasing

- **Unit 1** — engine receipt (one PR, ~200 lines).
- **Unit 2** — web surface (one PR).
- **Unit 3** — receipts (registry rows; the fluid-grid test either
  way).
- **Unit 4** — probe + adjudication (probe + decision-log entry).
- **Close-out** — RFC-072 Phase 3 marked COMPLETE-as-scoped with the
  pointer here; `status.md` row.

## Decision log

- *2026-08-27 — RFC opened on the browser-entry-point probe.* Scoped
  from the Phase-3 recon: the surface is a defect (the receipt hides
  the export), the fluid grid is a receipt-or-refuse, heterogeneity is
  a number, the library is parked under K67-3. RFC-073's Phase 0 census
  (same day) is the precedent for measuring before building.
